use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const MAX_OUTPUT_LINES: usize = 800;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pane {
    Commands,
    Settings,
    Output,
}

#[derive(Clone, Copy, Debug)]
pub enum PendingAction {
    RunCommand(usize),
    SaveAndRestart,
}

#[derive(Clone, Debug)]
pub struct ServiceAction {
    pub label: &'static str,
    pub command: &'static str,
    pub description: &'static str,
    pub confirm: bool,
}

impl ServiceAction {
    pub fn is_destructive(&self) -> bool {
        self.confirm
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsDoc {
    #[serde(default)]
    pub features: BTreeMap<String, bool>,
    #[serde(default)]
    pub layout: LayoutSection,
    #[serde(default)]
    pub keyboard: KeyboardSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSection {
    #[serde(default = "default_layout")]
    pub optspec_layout: String,
}

impl Default for LayoutSection {
    fn default() -> Self {
        Self {
            optspec_layout: default_layout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyboardSection {
    #[serde(default)]
    pub override_type: Option<String>,
}

#[derive(Clone, Debug)]
pub enum SettingEntry {
    LayoutOptspec,
    KeyboardOverride,
    Feature(String),
}

fn default_layout() -> String {
    "ABC".to_string()
}

pub struct App {
    pub focused_pane: Pane,
    pub commands: Vec<ServiceAction>,
    pub command_index: usize,
    pub setting_entries: Vec<SettingEntry>,
    pub setting_index: usize,
    pub settings: SettingsDoc,
    pub settings_path: PathBuf,
    pub service_ctl: PathBuf,
    pub service_state: String,
    pub status: String,
    pub output: Vec<String>,
    pub output_scroll: usize,
    pub confirm_prompt: Option<String>,
    pub pending_action: Option<PendingAction>,
    last_service_poll: Instant,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let home = home_dir()?;
        let settings_path = home.join(".config/keyrs/settings.toml");
        let service_ctl = resolve_service_ctl(&home);

        let mut settings = load_settings(&settings_path)?;
        ensure_settings_defaults(&mut settings);

        Ok(Self {
            focused_pane: Pane::Commands,
            commands: build_service_actions(&service_ctl),
            command_index: 0,
            setting_entries: build_setting_entries(&settings.features),
            setting_index: 0,
            settings,
            settings_path,
            service_ctl,
            service_state: "unknown".to_string(),
            status: "Ready".to_string(),
            output: vec![],
            output_scroll: 0,
            confirm_prompt: None,
            pending_action: None,
            last_service_poll: Instant::now() - Duration::from_secs(10),
        })
    }

    pub fn selected_setting(&self) -> Option<&SettingEntry> {
        self.setting_entries.get(self.setting_index)
    }

    pub fn selected_command(&self) -> Option<&ServiceAction> {
        self.commands.get(self.command_index)
    }

    pub fn set_status<S: Into<String>>(&mut self, msg: S) {
        self.status = msg.into();
    }

    pub fn push_output<S: AsRef<str>>(&mut self, msg: S) {
        for line in msg.as_ref().lines() {
            self.output.push(line.to_string());
        }
        if self.output.len() > MAX_OUTPUT_LINES {
            let overflow = self.output.len() - MAX_OUTPUT_LINES;
            self.output.drain(0..overflow);
        }
        // Auto-scroll to bottom on new output
        self.output_scroll = self.output.len().saturating_sub(1);
    }

    pub fn start_confirm<S: Into<String>>(&mut self, prompt: S, action: PendingAction) {
        self.confirm_prompt = Some(prompt.into());
        self.pending_action = Some(action);
    }

    pub fn clear_confirm(&mut self) {
        self.confirm_prompt = None;
        self.pending_action = None;
    }

    pub fn refresh_service_status(&mut self, force: bool) {
        if !force && self.last_service_poll.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.last_service_poll = Instant::now();
        self.service_state = query_service_state();
    }

    pub fn run_selected_command(&mut self) {
        let idx = self.command_index;
        if let Some(action) = self.commands.get(idx) {
            if action.confirm {
                self.start_confirm(
                    format!("Run '{}' ?", action.label),
                    PendingAction::RunCommand(idx),
                );
            } else {
                self.run_command_index(idx);
            }
        }
    }

    pub fn run_command_index(&mut self, index: usize) {
        let Some(action) = self.commands.get(index) else {
            return;
        };
        let label = action.label;
        let command = action.command;
        self.set_status(format!("Running {}...", label));

        match run_service_command(&self.service_ctl, command) {
            Ok(out) => {
                self.set_status(format!("{} succeeded", label));
                if !out.trim().is_empty() {
                    self.push_output(format!("$ {} --yes\n{}", command, out.trim_end()));
                }
            }
            Err(err) => {
                self.set_status(format!("{} failed", label));
                self.push_output(format!("$ {} --yes\n{}", command, err.trim_end()));
            }
        }
        self.refresh_service_status(true);
    }

    pub fn toggle_selected_feature(&mut self) {
        if let Some(SettingEntry::Feature(key)) = self.selected_setting().cloned() {
            let next = {
                let value = self.settings.features.entry(key.clone()).or_insert(false);
                *value = !*value;
                *value
            };
            self.set_status(format!("Toggled {} -> {}", key, next));
        }
    }

    pub fn cycle_layout(&mut self) {
        self.settings.layout.optspec_layout =
            if self.settings.layout.optspec_layout.eq_ignore_ascii_case("ABC") {
                "US".to_string()
            } else {
                "ABC".to_string()
            };
        self.set_status(format!(
            "layout.optspec_layout = {}",
            self.settings.layout.optspec_layout
        ));
    }

    pub fn cycle_keyboard_override(&mut self) {
        self.settings.keyboard.override_type =
            cycle_keyboard_override(self.settings.keyboard.override_type.as_deref());
        let v = self
            .settings
            .keyboard
            .override_type
            .as_deref()
            .unwrap_or("auto");
        self.set_status(format!("keyboard.override_type = {}", v));
    }

    pub fn change_selected_setting(&mut self) {
        match self.selected_setting() {
            Some(SettingEntry::LayoutOptspec) => self.cycle_layout(),
            Some(SettingEntry::KeyboardOverride) => self.cycle_keyboard_override(),
            Some(SettingEntry::Feature(_)) => self.toggle_selected_feature(),
            None => {}
        }
    }

    pub fn save_settings(&mut self, restart: bool) {
        match save_settings_atomic(&self.settings_path, &self.settings) {
            Ok(()) => {
                self.set_status("settings.toml saved");
                self.push_output(format!("Saved {}", self.settings_path.display()));
                if restart {
                    self.set_status("Saved settings, restarting service...");
                    match run_service_command(&self.service_ctl, "restart") {
                        Ok(out) => {
                            self.set_status("Saved settings and restarted service");
                            if !out.trim().is_empty() {
                                self.push_output(format!("$ restart --yes\n{}", out.trim_end()));
                            }
                        }
                        Err(err) => {
                            self.set_status("Settings saved, restart failed");
                            self.push_output(format!("$ restart --yes\n{}", err.trim_end()));
                        }
                    }
                    self.refresh_service_status(true);
                }
            }
            Err(err) => {
                self.set_status("Save failed");
                self.push_output(format!("Save error: {}", err));
            }
        }
    }

    pub fn cycle_pane_forward(&mut self) {
        self.focused_pane = match self.focused_pane {
            Pane::Commands => Pane::Settings,
            Pane::Settings => Pane::Output,
            Pane::Output => Pane::Commands,
        };
    }

    pub fn cycle_pane_backward(&mut self) {
        self.focused_pane = match self.focused_pane {
            Pane::Commands => Pane::Output,
            Pane::Settings => Pane::Commands,
            Pane::Output => Pane::Settings,
        };
    }
}

fn home_dir() -> io::Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))
}

fn resolve_service_ctl(home: &Path) -> PathBuf {
    let runtime = home.join(".local/bin/keyrs-service");
    if runtime.exists() {
        return runtime;
    }
    let repo_script = PathBuf::from("scripts/keyrs-service.sh");
    if repo_script.exists() {
        return repo_script;
    }
    PathBuf::from("keyrs-service")
}

fn load_settings(path: &Path) -> io::Result<SettingsDoc> {
    if !path.exists() {
        return Ok(SettingsDoc::default());
    }
    let content = fs::read_to_string(path)?;
    toml::from_str::<SettingsDoc>(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn save_settings_atomic(path: &Path, settings: &SettingsDoc) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("toml.tmp");
    let rendered = toml::to_string_pretty(settings)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(&temp, rendered)?;
    fs::rename(&temp, path)?;
    Ok(())
}

fn preferred_feature_order() -> Vec<&'static str> {
    vec![
        "Enter2Ent_Cmd",
        "Caps2Esc_Cmd",
        "Caps2Cmd",
        "forced_numpad",
        "media_arrows_fix",
        "multi_lang",
        "DistroFedoraGnome",
        "DistroPop",
        "DistroUbuntuOrFedoraGnome",
        "DesktopBudgie",
        "DesktopCosmicOrPop",
        "DesktopGnome",
        "DesktopKde",
        "DesktopPantheon",
        "DesktopSway",
        "DesktopXfce",
    ]
}

fn ensure_settings_defaults(settings: &mut SettingsDoc) {
    for key in preferred_feature_order() {
        settings
            .features
            .entry(key.to_string())
            .or_insert(false);
    }
    if settings.layout.optspec_layout.is_empty() {
        settings.layout.optspec_layout = default_layout();
    }
}

pub fn sorted_feature_keys(features: &BTreeMap<String, bool>) -> Vec<String> {
    let order = preferred_feature_order();
    let mut rank = BTreeMap::new();
    for (i, key) in order.iter().enumerate() {
        rank.insert(*key, i);
    }

    let mut keys: Vec<String> = features.keys().cloned().collect();
    keys.sort_by(|a, b| {
        let ra = rank.get(a.as_str());
        let rb = rank.get(b.as_str());
        match (ra, rb) {
            (Some(ra), Some(rb)) => ra.cmp(rb),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(b),
        }
    });
    keys
}

fn build_setting_entries(features: &BTreeMap<String, bool>) -> Vec<SettingEntry> {
    let mut out = vec![SettingEntry::LayoutOptspec, SettingEntry::KeyboardOverride];
    for key in sorted_feature_keys(features) {
        out.push(SettingEntry::Feature(key));
    }
    out
}

fn cycle_keyboard_override(current: Option<&str>) -> Option<String> {
    let values = [None, Some("IBM"), Some("Chromebook"), Some("Windows"), Some("Mac")];
    let mut idx = 0;
    for (i, v) in values.iter().enumerate() {
        let matches = match (v, current) {
            (None, None) => true,
            (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        };
        if matches {
            idx = i;
            break;
        }
    }
    values[(idx + 1) % values.len()].map(|s| s.to_string())
}

fn supports_install_commands(service_ctl: &Path) -> bool {
    let Ok(output) = Command::new(service_ctl).arg("--help").output() else {
        return false;
    };
    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text.contains("install") && text.contains("uninstall")
}

fn build_service_actions(service_ctl: &Path) -> Vec<ServiceAction> {
    let mut actions = vec![
        ServiceAction {
            label: "Status",
            command: "status",
            description: "Show keyrs service status",
            confirm: false,
        },
        ServiceAction {
            label: "Start",
            command: "start",
            description: "Start keyrs service",
            confirm: false,
        },
        ServiceAction {
            label: "Stop",
            command: "stop",
            description: "Stop keyrs service",
            confirm: true,
        },
        ServiceAction {
            label: "Restart",
            command: "restart",
            description: "Restart keyrs service",
            confirm: true,
        },
        ServiceAction {
            label: "Apply Config",
            command: "apply-config",
            description: "Compose, validate and replace config.toml",
            confirm: true,
        },
        ServiceAction {
            label: "Install Udev",
            command: "install-udev",
            description: "Install udev rules (sudo/root required)",
            confirm: true,
        },
        ServiceAction {
            label: "Uninstall Udev",
            command: "uninstall-udev",
            description: "Remove udev rules (sudo/root required)",
            confirm: true,
        },
    ];

    if supports_install_commands(service_ctl) {
        actions.push(ServiceAction {
            label: "Install Service",
            command: "install",
            description: "Install service/runtime assets",
            confirm: true,
        });
        actions.push(ServiceAction {
            label: "Uninstall Service",
            command: "uninstall",
            description: "Disable/remove user service unit",
            confirm: true,
        });
    }

    actions
}

fn run_service_command(service_ctl: &Path, command: &str) -> Result<String, String> {
    let output = Command::new(service_ctl)
        .arg(command)
        .arg("--yes")
        .output()
        .map_err(|e| format!("spawn error: {}", e))?;

    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    if output.status.success() {
        Ok(text)
    } else {
        Err(text)
    }
}

fn query_service_state() -> String {
    match Command::new("systemctl")
        .arg("--user")
        .arg("is-active")
        .arg("keyrs.service")
        .output()
    {
        Ok(out) => {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if state.is_empty() {
                if out.status.success() {
                    "active".to_string()
                } else {
                    "inactive".to_string()
                }
            } else {
                state
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

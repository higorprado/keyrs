// Keyrs Config Parser - TOML with Serde
// Parses configuration from TOML files

#[cfg(feature = "pure-rust")]
use std::collections::HashMap;
#[cfg(feature = "pure-rust")]
use std::fs;
#[cfg(feature = "pure-rust")]
use std::path::Path;
#[cfg(feature = "pure-rust")]
use std::sync::OnceLock;

use crate::mapping::{ActionStep, Keymap, KeymapValue, Modmap, MultiModmap};
use crate::{Combo, ComboHint, Key, Modifier};
use serde::Deserialize;

#[cfg(feature = "pure-rust")]
fn config_debug_enabled() -> bool {
    static DEBUG_CONFIG: OnceLock<bool> = OnceLock::new();
    *DEBUG_CONFIG.get_or_init(|| {
        std::env::var("KEYRS_DEBUG_CONFIG")
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(false)
    })
}

/// Configuration parser errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid modifier: {0}")]
    InvalidModifier(String),

    #[error("Invalid combo string: {0}")]
    InvalidCombo(String),

    #[error("Timeout value out of range: {0}")]
    TimeoutOutOfRange(String),
}

/// Main configuration structure (root TOML table)
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigToml {
    /// General settings
    #[serde(default)]
    pub general: Option<GeneralConfig>,

    /// Modmaps configuration
    #[serde(default)]
    pub modmap: ModmapConfig,

    /// Multipurpose modmaps (tap/hold keys)
    #[serde(default)]
    pub multipurpose: Vec<MultipurposeTomlEntry>,

    /// Keymaps configuration
    #[serde(default)]
    pub keymap: Vec<KeymapTomlEntry>,

    /// Timeouts configuration
    #[serde(default)]
    pub timeouts: Option<TimeoutConfig>,

    /// Device filter configuration
    #[serde(default)]
    pub devices: Option<DevicesConfig>,

    /// Output throttle delays
    #[serde(default)]
    pub delays: Option<DelayConfig>,

    // Main event loop and window polling behavior
    #[serde(default)]
    pub window: Option<WindowConfig>,
}

/// General settings
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneralConfig {
    /// Suspend key name
    pub suspend_key: Option<String>,
    /// Diagnostics dump key name
    pub diagnostics_key: Option<String>,
    /// Emergency eject key name
    pub emergency_eject_key: Option<String>,
}

/// Device filtering configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevicesConfig {
    /// Explicit device names/paths to use
    #[serde(default)]
    pub only: Vec<String>,
}

/// Modmap configuration (supports default and conditional modmaps)
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ModmapConfig {
    /// Default modmap (applies to all windows)
    pub default: Option<HashMap<String, String>>,

    /// Conditional modmaps (window-specific)
    #[serde(default)]
    pub conditionals: Vec<ConditionalModmap>,
}

/// Conditional modmap entry
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalModmap {
    /// Name identifier for this modmap
    pub name: String,

    /// Key mappings for this conditional
    pub mappings: HashMap<String, String>,

    /// Condition string (regex on wm_class or wm_name)
    pub condition: String,
}

/// Multipurpose modmap entry (tap/hold behavior)
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultipurposeTomlEntry {
    /// Name identifier for this multipurpose modmap
    pub name: String,

    /// Trigger key (the key being remapped)
    pub trigger: String,

    /// Output key for tap (short press)
    pub tap: String,

    /// Output key for hold (long press)
    pub hold: String,

    /// Optional condition string (window-specific)
    pub condition: Option<String>,
}

/// Keymap entry (can be array of tables or single table)
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeymapTomlEntry {
    /// Optional name for this keymap
    pub name: Option<String>,

    /// Combo-to-output mappings
    #[serde(default)]
    pub mappings: HashMap<String, KeymapTomlOutput>,

    /// Optional condition string (window-specific)
    pub condition: Option<String>,
}

/// Output side of a keymap entry (supports various formats)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum KeymapTomlOutput {
    /// Single key as string
    Single(String),

    /// List of outputs (for sequences)
    Multiple(Vec<String>),
}

/// Timeout configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeoutConfig {
    /// Multipurpose modmap timeout (milliseconds)
    pub multipurpose: Option<u64>,

    /// Suspend timeout (milliseconds)
    pub suspend: Option<u64>,
}

/// Output delay configuration (milliseconds)
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DelayConfig {
    /// Delay before key output
    pub key_pre_delay_ms: Option<u64>,
    /// Delay after key output
    pub key_post_delay_ms: Option<u64>,
}

/// Main loop / window polling configuration (milliseconds)
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowConfig {
    /// Timeout passed to evdev poll loop
    pub poll_timeout_ms: Option<u64>,
    /// Interval between window context refreshes
    pub update_interval_ms: Option<u64>,
    /// Sleep duration after a no-event poll error path
    pub idle_sleep_ms: Option<u64>,
}

// Use TimeoutConfig directly (serde handles both singular and plural)
// The #[serde(default)] attribute makes both forms work

/// Main configuration structure
#[derive(Debug, Clone)]
pub struct Config {
    /// Modmaps (first is default, rest are conditional)
    pub modmaps: Vec<ModmapEntry>,
    /// Multipurpose modmaps (tap/hold behavior)
    pub multipurpose: Vec<MultipurposeEntry>,
    /// Keymaps
    pub keymaps: Vec<KeymapEntry>,
    /// Optional suspend key
    pub suspend_key: Option<Key>,
    /// Multipurpose key timeout (milliseconds)
    pub multipurpose_timeout: Option<u64>,
    /// Suspend timeout (milliseconds)
    pub suspend_timeout: Option<u64>,
    /// Diagnostics key (optional)
    pub diagnostics_key: Option<Key>,
    /// Emergency eject key (optional)
    pub emergency_eject_key: Option<Key>,
    /// Device name/path filter (empty = autodetect keyboards)
    pub device_filter: Vec<String>,
    /// Pre-key output delay in milliseconds
    pub key_pre_delay_ms: Option<u64>,
    /// Post-key output delay in milliseconds
    pub key_post_delay_ms: Option<u64>,
    // Event poll timeout in milliseconds
    pub poll_timeout_ms: Option<u64>,
    // Window context refresh interval in milliseconds
    pub window_update_interval_ms: Option<u64>,
    // Idle loop sleep in milliseconds
    pub idle_sleep_ms: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            modmaps: vec![],
            multipurpose: vec![],
            keymaps: vec![],
            suspend_key: None,
            multipurpose_timeout: None,
            suspend_timeout: None,
            diagnostics_key: None,
            emergency_eject_key: None,
            device_filter: vec![],
            key_pre_delay_ms: None,
            key_post_delay_ms: None,
            poll_timeout_ms: None,
            window_update_interval_ms: None,
            idle_sleep_ms: None,
        }
    }
}

/// Multipurpose modmap entry for internal use
#[derive(Debug, Clone)]
pub struct MultipurposeEntry {
    /// Name identifier
    pub name: String,
    /// Trigger key
    pub trigger: Key,
    /// Tap output key
    pub tap: Key,
    /// Hold output key
    pub hold: Key,
    /// Optional condition
    pub condition: Option<String>,
}

impl Config {
    /// Parse a TOML configuration file
    #[cfg(feature = "pure-rust")]
    pub fn from_toml_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Parse configuration from TOML string
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        // Parse TOML
        let toml_config: ConfigToml =
            toml::from_str(content).map_err(|e| ConfigError::TomlParse(e.to_string()))?;

        // Convert to internal Config
        toml_config.to_config()
    }

    /// Convert to TransformConfig for use in TransformEngine
    pub fn to_transform_config(&self) -> TransformConfig {
        use std::collections::HashMap;

        log::debug!("to_transform_config called with {} keymaps", self.keymaps.len());

        TransformConfig {
            modmaps: self
                .modmaps
                .iter()
                .map(|entry| {
                    let mut mappings = HashMap::new();
                    for (from, to) in &entry.mappings {
                        mappings.insert(*from, *to);
                    }
                    if let Some(condition) = &entry.condition {
                        Modmap::with_conditional(&entry.name, mappings, condition.clone())
                    } else {
                        Modmap::new(&entry.name, mappings)
                    }
                })
                .collect(),
            multimodmaps: self
                .multipurpose
                .iter()
                .map(|entry| {
                    let mut mappings = HashMap::new();
                    mappings.insert(entry.trigger, (entry.tap, entry.hold));
                    if let Some(condition) = &entry.condition {
                        MultiModmap::with_conditional(&entry.name, mappings, condition.clone())
                    } else {
                        MultiModmap::new(&entry.name, mappings)
                    }
                })
                .collect(),
            keymaps: self
                .keymaps
                .iter()
                .map(|entry| {
                    let mut mappings = HashMap::new();
                    for (combo_str, output) in &entry.mappings {
                        // Parse combo string
                        match super::parse_combo_string(combo_str) {
                            Ok(parsed) => {
                                let combo = Combo::new(parsed.modifiers, parsed.key);
                                let value: KeymapValue = output.clone().into();
                                mappings.insert(combo, value);
                            }
                            Err(e) => {
                                log::warn!(
                                    "Failed to parse input combo '{}' in keymap '{}': {}",
                                    combo_str, entry.name, e
                                );
                            }
                        }
                    }

                    log::debug!(
                        "Keymap '{}' converted with {} mappings",
                        entry.name,
                        mappings.len()
                    );

                    if let Some(condition) = &entry.condition {
                        Keymap::with_conditional(&entry.name, mappings, condition.clone())
                    } else {
                        Keymap::with_mappings(&entry.name, mappings)
                    }
                })
                .collect(),
            suspend_key: self.suspend_key,
            multipurpose_timeout: self.multipurpose_timeout,
            suspend_timeout: self.suspend_timeout,
        }
    }
}

impl ConfigToml {
    /// Convert parsed TOML to internal Config structure
    fn to_config(&self) -> Result<Config, ConfigError> {
        let mut config = Config::default();

        // Parse suspend key
        if let Some(general) = &self.general {
            if let Some(key_str) = &general.suspend_key {
                config.suspend_key = Some(parse_key(key_str)?);
            }
            if let Some(key_str) = &general.diagnostics_key {
                config.diagnostics_key = Some(parse_key(key_str)?);
            }
            if let Some(key_str) = &general.emergency_eject_key {
                config.emergency_eject_key = Some(parse_key(key_str)?);
            }
        }

        // Parse default modmap
        if let Some(default_mappings) = &self.modmap.default {
            let mut mappings = HashMap::new();
            for (from_str, to_str) in default_mappings {
                let from_key = parse_key(from_str)?;
                let to_key = parse_key(to_str)?;
                mappings.insert(from_key, to_key);
            }
            config.modmaps.push(ModmapEntry {
                name: "default".to_string(),
                mappings: mappings.into_iter().collect(),
                condition: None,
            });
        }

        // Parse conditional modmaps
        for conditional in &self.modmap.conditionals {
            let mut mappings = HashMap::new();
            for (from_str, to_str) in &conditional.mappings {
                let from_key = parse_key(from_str)?;
                let to_key = parse_key(to_str)?;
                mappings.insert(from_key, to_key);
            }
            config.modmaps.push(ModmapEntry {
                name: conditional.name.clone(),
                mappings: mappings.into_iter().collect(),
                condition: Some(conditional.condition.clone()),
            });
        }

        // Parse multipurpose modmaps
        for mp_entry in &self.multipurpose {
            let trigger = parse_key(&mp_entry.trigger)?;
            let tap = parse_key(&mp_entry.tap)?;
            let hold = parse_key(&mp_entry.hold)?;
            config.multipurpose.push(MultipurposeEntry {
                name: mp_entry.name.clone(),
                trigger,
                tap,
                hold,
                condition: mp_entry.condition.clone(),
            });
        }

        // Parse keymaps
        for keymap_entry in &self.keymap {
            let mut mappings = HashMap::new();
            let keymap_name = keymap_entry.name.clone().unwrap_or_else(|| {
                format!(
                    "keymap_{}",
                    keymap_entry
                        .mappings
                        .keys()
                        .next()
                        .unwrap_or(&"unnamed".to_string())
                )
            });

            for (combo_str, output) in &keymap_entry.mappings {
                match output {
                    KeymapTomlOutput::Single(s) => {
                        if let Some(text) = parse_text_output(s) {
                            mappings.insert(combo_str.clone(), KeymapOutput::Text(text));
                            continue;
                        }
                        if let Some(codepoint) = parse_unicode_output(s) {
                            mappings.insert(combo_str.clone(), KeymapOutput::Unicode(codepoint));
                            continue;
                        }

                        // Try parsing as a combo first (e.g., "Ctrl-c" or "Ctrl-Shift-c")
                        match super::parse_combo_string(s) {
                            Ok(parsed) => {
                                // Output is a combo - convert to sequence of keys
                                let mut keys = Vec::new();

                                // Add modifier keys (use first key from each modifier)
                                for modifier in &parsed.modifiers {
                                    if let Some(&first_key) = modifier.keys().first() {
                                        keys.push(first_key);
                                    }
                                }

                                // Add the final key
                                keys.push(parsed.key);

                                mappings.insert(combo_str.clone(), KeymapOutput::Combo(keys));
                            }
                            Err(e) => {
                                // Try parsing as a single key instead
                                match parse_key(s) {
                                    Ok(key) => {
                                        mappings.insert(combo_str.clone(), KeymapOutput::Key(key));
                                    }
                                    Err(_) => {
                                        log::warn!(
                                            "Failed to parse keymap output '{}' in keymap '{}': {}",
                                            s, keymap_name, e
                                        );
                                    }
                                }
                            }
                        }
                    }
                    KeymapTomlOutput::Multiple(list) => {
                        let keys: Vec<Key> = list.iter().filter_map(|s| parse_key(s).ok()).collect();
                        if keys.len() == list.len() {
                            mappings.insert(combo_str.clone(), KeymapOutput::Combo(keys));
                            continue;
                        }

                        let mut steps = Vec::with_capacity(list.len());
                        let mut invalid = false;
                        for item in list {
                            if let Some(step) = parse_sequence_step(item) {
                                steps.push(step);
                            } else {
                                invalid = true;
                                log::warn!(
                                    "Invalid sequence step '{}' in keymap '{}' mapping '{}'",
                                    item, keymap_name, combo_str
                                );
                            }
                        }

                        if !invalid && !steps.is_empty() {
                            mappings.insert(combo_str.clone(), KeymapOutput::Sequence(steps));
                        } else {
                            log::warn!(
                                "Invalid sequence in keymap '{}' mapping '{}'",
                                keymap_name, combo_str
                            );
                        }
                    }
                }
            }

            log::debug!(
                "Loaded keymap '{}' with {} mappings, conditional={}",
                keymap_name,
                mappings.len(),
                keymap_entry.condition.is_some()
            );

            #[cfg(feature = "pure-rust")]
            if config_debug_enabled() {
                for (combo, output) in &mappings {
                    log::trace!(
                        "keymap='{}' combo='{}' output={:?}",
                        keymap_name, combo, output
                    );
                }
            }

            config.keymaps.push(KeymapEntry {
                name: keymap_name,
                mappings: mappings.into_iter().collect(),
                condition: keymap_entry.condition.clone(),
            });
        }

        // Parse timeouts
        if let Some(timeouts) = &self.timeouts {
            if let Some(mp) = timeouts.multipurpose {
                if mp < 100 || mp > 5000 {
                    return Err(ConfigError::TimeoutOutOfRange(format!(
                        "multipurpose must be 100-5000ms, got {}",
                        mp
                    )));
                }
                config.multipurpose_timeout = Some(mp);
            }
            if let Some(st) = timeouts.suspend {
                if st < 100 || st > 10000 {
                    return Err(ConfigError::TimeoutOutOfRange(format!(
                        "suspend must be 100-10000ms, got {}",
                        st
                    )));
                }
                config.suspend_timeout = Some(st);
            }
        }

        // Parse devices
        if let Some(devices) = &self.devices {
            config.device_filter = devices.only.clone();
        }

        // Parse output delays
        if let Some(delays) = &self.delays {
            if let Some(pre) = delays.key_pre_delay_ms {
                if pre > 150 {
                    return Err(ConfigError::TimeoutOutOfRange(format!(
                        "key_pre_delay_ms must be 0-150ms, got {}",
                        pre
                    )));
                }
                config.key_pre_delay_ms = Some(pre);
            }
            if let Some(post) = delays.key_post_delay_ms {
                if post > 150 {
                    return Err(ConfigError::TimeoutOutOfRange(format!(
                        "key_post_delay_ms must be 0-150ms, got {}",
                        post
                    )));
                }
                config.key_post_delay_ms = Some(post);
            }
        }

        // Parse window loop timing controls
        if let Some(window) = &self.window {
            if let Some(poll) = window.poll_timeout_ms {
                if poll == 0 || poll > 5000 {
                    return Err(ConfigError::TimeoutOutOfRange(format!(
                        "window.poll_timeout_ms must be 1-5000ms, got {}",
                        poll
                    )));
                }
                config.poll_timeout_ms = Some(poll);
            }
            if let Some(update) = window.update_interval_ms {
                if update < 10 || update > 10000 {
                    return Err(ConfigError::TimeoutOutOfRange(format!(
                        "window.update_interval_ms must be 10-10000ms, got {}",
                        update
                    )));
                }
                config.window_update_interval_ms = Some(update);
            }
            if let Some(idle) = window.idle_sleep_ms {
                if idle > 1000 {
                    return Err(ConfigError::TimeoutOutOfRange(format!(
                        "window.idle_sleep_ms must be 0-1000ms, got {}",
                        idle
                    )));
                }
                config.idle_sleep_ms = Some(idle);
            }
        }

        Ok(config)
    }
}

/// Configuration entry for a modmap
#[derive(Debug, Clone)]
pub struct ModmapEntry {
    /// Name of modmap
    pub name: String,
    /// Key mappings (from_key -> to_key)
    pub mappings: Vec<(Key, Key)>,
    /// Optional condition (for conditional modmaps)
    pub condition: Option<String>,
}

/// Configuration entry for a keymap
#[derive(Debug, Clone)]
pub struct KeymapEntry {
    /// Name of keymap
    pub name: String,
    /// Combo mappings (combo_str -> output)
    pub mappings: Vec<(String, KeymapOutput)>,
    /// Optional window condition
    pub condition: Option<String>,
}

/// Output side of a keymap entry
#[derive(Debug, Clone)]
pub enum KeymapOutput {
    Key(Key),
    Combo(Vec<Key>),
    Sequence(Vec<ActionStep>),
    ComboHint(ComboHint),
    Unicode(u32),
    Text(String),
}

impl From<Key> for KeymapOutput {
    fn from(key: Key) -> Self {
        KeymapOutput::Key(key)
    }
}

impl From<KeymapTomlOutput> for KeymapOutput {
    fn from(value: KeymapTomlOutput) -> Self {
        match value {
            KeymapTomlOutput::Single(s) => {
                // Try parsing as key, then combo hint, then combo
                if let Ok(key) = parse_key(&s) {
                    KeymapOutput::Key(key)
                } else if let Some(codepoint) = parse_unicode_output(&s) {
                    KeymapOutput::Unicode(codepoint)
                } else if let Some(text) = parse_text_output(&s) {
                    KeymapOutput::Text(text)
                } else if let Ok(hint) = parse_combo_hint(&s) {
                    KeymapOutput::ComboHint(hint)
                } else {
                    // Invalid, but we'll store as string and handle at runtime
                    KeymapOutput::Key(Key::from(0))
                }
            }
            KeymapTomlOutput::Multiple(list) => {
                let keys: Vec<Key> = list.iter().filter_map(|s| parse_key(s).ok()).collect();
                if keys.len() == list.len() {
                    KeymapOutput::Combo(keys)
                } else {
                    let steps: Vec<ActionStep> =
                        list.iter().filter_map(|s| parse_sequence_step(s)).collect();
                    KeymapOutput::Sequence(steps)
                }
            }
        }
    }
}

impl Into<KeymapValue> for KeymapOutput {
    fn into(self) -> KeymapValue {
        match self {
            KeymapOutput::Key(k) => KeymapValue::Key(k),
            KeymapOutput::Combo(keys) => {
                // Reconstruct combo from key sequence
                // The vec should be [modifier_key1, modifier_key2, ..., final_key]
                if keys.is_empty() {
                    KeymapValue::Key(Key::from(0))
                } else if keys.len() == 1 {
                    // Just a single key
                    KeymapValue::Key(keys[0])
                } else {
                    // Multiple keys: treat all but last as modifiers, last as the key
                    let final_key = keys[keys.len() - 1];
                    let modifier_keys = &keys[..keys.len() - 1];

                    // Convert key codes to Modifiers
                    let modifiers: Vec<Modifier> = modifier_keys
                        .iter()
                        .filter_map(|k| Modifier::from_key(*k))
                        .collect();

                    let combo = Combo::new(modifiers, final_key);
                    KeymapValue::Combo(combo)
                }
            }
            KeymapOutput::Sequence(steps) => KeymapValue::Sequence(steps),
            KeymapOutput::ComboHint(h) => KeymapValue::ComboHint(h),
            KeymapOutput::Unicode(codepoint) => KeymapValue::Unicode(codepoint),
            KeymapOutput::Text(text) => KeymapValue::Text(text),
        }
    }
}

/// Configuration for transform engine
pub use crate::transform::TransformConfig;

/// Parse a key name into a Key
fn parse_key(name: &str) -> Result<Key, ConfigError> {
    let trimmed = name.trim();
    crate::key::key_from_name(trimmed).ok_or_else(|| ConfigError::InvalidKey(trimmed.to_string()))
}

/// Parse Unicode output syntax.
///
/// Supported formats:
/// - `U+00E9`
/// - `Unicode(00E9)`
/// - `Unicode(0x00E9)`
fn parse_unicode_output(s: &str) -> Option<u32> {
    let trimmed = s.trim();

    let parse_hex = |hex: &str| -> Option<u32> {
        let hex = hex.trim();
        let hex = hex
            .strip_prefix("0x")
            .or_else(|| hex.strip_prefix("0X"))
            .unwrap_or(hex);
        u32::from_str_radix(hex, 16).ok()
    };

    if let Some(hex) = trimmed.strip_prefix("U+") {
        return parse_hex(hex);
    }

    if trimmed.len() >= 9 && trimmed[..8].eq_ignore_ascii_case("unicode(") && trimmed.ends_with(')')
    {
        let inner = &trimmed[8..trimmed.len() - 1];
        return parse_hex(inner);
    }

    None
}

/// Parse text output syntax.
///
/// Supported formats:
/// - `Text(hello)`
/// - `text(Hello world)`
/// - `Text("Hello world")`
/// - `Text('Hello world')`
fn parse_text_output(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.len() < 6 {
        return None;
    }
    if !trimmed[..5].eq_ignore_ascii_case("text(") || !trimmed.ends_with(')') {
        return None;
    }

    let inner = trimmed[5..trimmed.len() - 1].trim();
    let unquoted = inner
        .strip_prefix('"')
        .and_then(|x| x.strip_suffix('"'))
        .or_else(|| inner.strip_prefix('\'').and_then(|x| x.strip_suffix('\'')))
        .unwrap_or(inner);

    Some(unquoted.to_string())
}

fn parse_delay_step(s: &str) -> Option<u64> {
    let trimmed = s.trim();
    if trimmed.len() < 8 {
        return None;
    }
    if !trimmed[..6].eq_ignore_ascii_case("delay(") || !trimmed.ends_with(')') {
        return None;
    }
    let inner = &trimmed[6..trimmed.len() - 1];
    inner.trim().parse::<u64>().ok()
}

fn parse_ignore_step(s: &str) -> bool {
    let trimmed = s.trim();
    trimmed.eq_ignore_ascii_case("ignore")
        || trimmed.eq_ignore_ascii_case("noop")
        || trimmed.eq_ignore_ascii_case("no_op")
}

fn parse_bind_step(s: &str) -> bool {
    let trimmed = s.trim();
    trimmed.eq_ignore_ascii_case("bind") || trimmed.eq_ignore_ascii_case("combo(bind)")
}

fn parse_set_setting_step(s: &str) -> Option<ActionStep> {
    let trimmed = s.trim();
    let lower = trimmed.to_ascii_lowercase();
    let inner = if lower.starts_with("setsetting(") && trimmed.ends_with(')') {
        &trimmed["SetSetting(".len()..trimmed.len() - 1]
    } else if lower.starts_with("set(") && trimmed.ends_with(')') {
        &trimmed["Set(".len()..trimmed.len() - 1]
    } else {
        return None;
    };

    let mut parts = inner.splitn(2, '=');
    let name = parts.next()?.trim();
    let value_raw = parts.next()?.trim().to_ascii_lowercase();
    if name.is_empty() {
        return None;
    }

    let value = match value_raw.as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => return None,
    };

    Some(ActionStep::SetSetting {
        name: name.to_string(),
        value,
    })
}

fn parse_combo_step(s: &str) -> Option<Combo> {
    let trimmed = s.trim();
    let combo_expr = if trimmed.len() >= 7
        && trimmed[..6].eq_ignore_ascii_case("combo(")
        && trimmed.ends_with(')')
    {
        trimmed[6..trimmed.len() - 1].trim()
    } else {
        trimmed
    };

    if let Ok(parsed) = super::parse_combo_string(combo_expr) {
        return Some(Combo::new(parsed.modifiers, parsed.key));
    }
    parse_key(combo_expr).ok().map(|k| Combo::new(Vec::new(), k))
}

fn parse_sequence_step(s: &str) -> Option<ActionStep> {
    if let Some(ms) = parse_delay_step(s) {
        return Some(ActionStep::DelayMs(ms));
    }
    if let Some(step) = parse_set_setting_step(s) {
        return Some(step);
    }
    if parse_bind_step(s) {
        return Some(ActionStep::Bind);
    }
    if parse_ignore_step(s) {
        return Some(ActionStep::Ignore);
    }
    if let Some(text) = parse_text_output(s) {
        return Some(ActionStep::Text(text));
    }
    parse_combo_step(s).map(ActionStep::Combo)
}

/// Parse a combo hint string (e.g., "combo(bind)")
fn parse_combo_hint(s: &str) -> Result<ComboHint, ConfigError> {
    let trimmed = s.trim().to_lowercase();

    // Match against known hints
    // Note: ComboHint enum uses Bind=1, EscapeNextKey=2, Ignore=3, EscapeNextCombo=4
    if trimmed == "combo(bind)" || trimmed == "bind" {
        Ok(ComboHint::Bind)
    } else if trimmed == "escape_next" || trimmed == "escapenext" || trimmed == "escape_next_key" {
        Ok(ComboHint::EscapeNextKey)
    } else if trimmed == "ignore" {
        Ok(ComboHint::Ignore)
    } else if trimmed == "escape_next_combo" {
        Ok(ComboHint::EscapeNextCombo)
    } else {
        Err(ConfigError::InvalidCombo(format!("unknown hint: {}", s)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key() {
        assert_eq!(parse_key("a").unwrap(), Key::from(30));
        assert_eq!(parse_key("left_ctrl").unwrap(), Key::from(29));
        assert!(parse_key("notakey").is_err());
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_config_from_simple_toml() {
        let toml = r#"
            [general]
            suspend_key = "capslock"

            [modmap.default]
            capslock = "left_ctrl"
            escape = "capslock"

            [[keymap]]
            name = "Emacs-like"
            [keymap.mappings]
            "ctrl-b" = "left"
            "ctrl-f" = "right"
        "#;

        let config = Config::from_toml(toml).unwrap();
        assert!(!config.modmaps.is_empty());
        assert!(!config.keymaps.is_empty());
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_config_with_conditionals() {
        let toml = r#"
            [modmap.default]
            capslock = "left_ctrl"

            [[modmap.conditionals]]
            name = "Emacs"
            condition = "wm_class =~ 'Emacs'"
            [modmap.conditionals.mappings]
            capslock = "left_ctrl"
        "#;

        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.modmaps.len(), 2);
        assert_eq!(config.modmaps[0].name, "default");
        assert_eq!(config.modmaps[1].name, "Emacs");
        assert!(config.modmaps[1].condition.is_some());
    }

    #[test]
    fn test_parse_combo_hint() {
        assert_eq!(parse_combo_hint("combo(bind)").unwrap(), ComboHint::Bind);
        assert_eq!(
            parse_combo_hint("escape_next").unwrap(),
            ComboHint::EscapeNextKey
        );
        assert_eq!(parse_combo_hint("ignore").unwrap(), ComboHint::Ignore);
        assert!(parse_combo_hint("unknown").is_err());
    }

    #[test]
    fn test_parse_unicode_output() {
        assert_eq!(parse_unicode_output("U+00E9"), Some(0x00E9));
        assert_eq!(parse_unicode_output("Unicode(00E9)"), Some(0x00E9));
        assert_eq!(parse_unicode_output("unicode(0x00E9)"), Some(0x00E9));
        assert_eq!(parse_unicode_output("é"), None);
        assert_eq!(parse_unicode_output("not-unicode"), None);
    }

    #[test]
    fn test_parse_text_output() {
        assert_eq!(parse_text_output("Text(hello)"), Some("hello".to_string()));
        assert_eq!(
            parse_text_output("text(\"hello world\")"),
            Some("hello world".to_string())
        );
        assert_eq!(parse_text_output("Unicode(00E9)"), None);
    }

    #[test]
    fn test_parse_sequence_step() {
        assert_eq!(parse_sequence_step("Delay(200)"), Some(ActionStep::DelayMs(200)));
        assert_eq!(
            parse_sequence_step("SetSetting(Enter2Ent_Cmd=true)"),
            Some(ActionStep::SetSetting {
                name: "Enter2Ent_Cmd".to_string(),
                value: true
            })
        );
        assert_eq!(
            parse_sequence_step("set(Enter2Ent_Cmd=off)"),
            Some(ActionStep::SetSetting {
                name: "Enter2Ent_Cmd".to_string(),
                value: false
            })
        );
        assert_eq!(parse_sequence_step("bind"), Some(ActionStep::Bind));
        assert_eq!(parse_sequence_step("Ignore"), Some(ActionStep::Ignore));
        assert_eq!(
            parse_sequence_step("Text(hello)"),
            Some(ActionStep::Text("hello".to_string()))
        );
        assert!(matches!(
            parse_sequence_step("Ctrl-t"),
            Some(ActionStep::Combo(_))
        ));
    }

    #[test]
    fn test_keymap_unicode_output_conversion() {
        use crate::mapping::KeymapValue;
        let value: KeymapValue = KeymapOutput::Unicode(0x00E9).into();
        assert_eq!(value, KeymapValue::Unicode(0x00E9));
    }

    #[test]
    fn test_keymap_text_output_conversion() {
        use crate::mapping::KeymapValue;
        let value: KeymapValue = KeymapOutput::Text("hello".to_string()).into();
        assert_eq!(value, KeymapValue::Text("hello".to_string()));
    }

    #[test]
    fn test_keymap_sequence_output_conversion() {
        use crate::mapping::KeymapValue;
        let value: KeymapValue = KeymapOutput::Sequence(vec![
            ActionStep::DelayMs(100),
            ActionStep::Text("x".to_string()),
        ])
        .into();
        assert_eq!(
            value,
            KeymapValue::Sequence(vec![
                ActionStep::DelayMs(100),
                ActionStep::Text("x".to_string())
            ])
        );
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_explicit_combo_sequence_list_parses_as_sequence() {
        let toml = r#"
            [[keymap]]
            name = "seq_combo_test"
            condition = "wm_class =~ 'terminal'"
            [keymap.mappings]
            "Alt-Delete" = ["Combo(Esc)", "Delay(25)", "Combo(d)"]
        "#;

        let config = Config::from_toml(toml).expect("config should parse");
        assert_eq!(config.keymaps.len(), 1);
        assert_eq!(config.keymaps[0].name, "seq_combo_test");

        let (_combo, output) = config.keymaps[0]
            .mappings
            .iter()
            .find(|(combo, _)| combo == "Alt-Delete")
            .expect("Alt-Delete mapping should exist");

        assert!(matches!(output, KeymapOutput::Sequence(steps) if steps.len() == 3));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_config_with_multipurpose() {
        let toml = r#"
            [[multipurpose]]
            name = "Caps2Esc"
            trigger = "capslock"
            tap = "escape"
            hold = "right_ctrl"

            [[multipurpose]]
            name = "Enter2Cmd"
            trigger = "enter"
            tap = "enter"
            hold = "right_ctrl"
            condition = "wm_class =~ 'Firefox'"
        "#;

        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.multipurpose.len(), 2);
        
        // Check first entry (Caps2Esc)
        assert_eq!(config.multipurpose[0].name, "Caps2Esc");
        assert_eq!(config.multipurpose[0].trigger, Key::from(58)); // CAPSLOCK
        assert_eq!(config.multipurpose[0].tap, Key::from(1)); // ESCAPE
        assert_eq!(config.multipurpose[0].hold, Key::from(97)); // RIGHT_CTRL
        assert!(config.multipurpose[0].condition.is_none());
        
        // Check second entry (Enter2Cmd with condition)
        assert_eq!(config.multipurpose[1].name, "Enter2Cmd");
        assert_eq!(config.multipurpose[1].trigger, Key::from(28)); // ENTER
        assert_eq!(config.multipurpose[1].tap, Key::from(28)); // ENTER
        assert_eq!(config.multipurpose[1].hold, Key::from(97)); // RIGHT_CTRL
        assert_eq!(config.multipurpose[1].condition, Some("wm_class =~ 'Firefox'".to_string()));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_config_full_with_multipurpose() {
        let toml = r#"
            [general]
            suspend_key = "f16"
            diagnostics_key = "f15"
            emergency_eject_key = "f17"

            [modmap.default]
            capslock = "escape"

            [[multipurpose]]
            name = "Caps2Esc"
            trigger = "capslock"
            tap = "escape"
            hold = "right_ctrl"

            [timeouts]
            multipurpose = 200
            suspend = 1000

            [devices]
            only = ["Telink Wireless Gaming Keyboard"]

            [delays]
            key_pre_delay_ms = 8
            key_post_delay_ms = 12

            [window]
            poll_timeout_ms = 120
            update_interval_ms = 450
            idle_sleep_ms = 7
        "#;

        let config = Config::from_toml(toml).unwrap();
        
        // Check suspend key
        assert_eq!(config.suspend_key, Some(Key::from(186))); // F16
        assert_eq!(config.diagnostics_key, Some(Key::from(185))); // F15
        assert_eq!(config.emergency_eject_key, Some(Key::from(187))); // F17
        
        // Check modmaps
        assert_eq!(config.modmaps.len(), 1);
        assert_eq!(config.modmaps[0].name, "default");
        
        // Check multipurpose
        assert_eq!(config.multipurpose.len(), 1);
        assert_eq!(config.multipurpose[0].name, "Caps2Esc");
        
        // Check timeouts
        assert_eq!(config.multipurpose_timeout, Some(200));
        assert_eq!(config.suspend_timeout, Some(1000));
        assert_eq!(config.device_filter, vec!["Telink Wireless Gaming Keyboard".to_string()]);
        assert_eq!(config.key_pre_delay_ms, Some(8));
        assert_eq!(config.key_post_delay_ms, Some(12));
        assert_eq!(config.poll_timeout_ms, Some(120));
        assert_eq!(config.window_update_interval_ms, Some(450));
        assert_eq!(config.idle_sleep_ms, Some(7));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_multipurpose_invalid_key() {
        let toml = r#"
            [[multipurpose]]
            name = "Invalid"
            trigger = "not_a_real_key"
            tap = "escape"
            hold = "right_ctrl"
        "#;

        let result = Config::from_toml(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid key"));
    }
}

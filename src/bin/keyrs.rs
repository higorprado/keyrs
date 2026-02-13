// Xwaykeyz Pure Rust CLI
// Standalone binary for Wayland key remapping without Python dependencies

#![cfg_attr(feature = "pure-rust", allow(dead_code))]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "pure-rust")]
use clap::Parser;
#[cfg(feature = "pure-rust")]
use toml::Value;

#[cfg(feature = "pure-rust")]
use keyrs_core::config::parser::Config;
#[cfg(feature = "pure-rust")]
use keyrs_core::output::TransformResultOutput;
#[cfg(feature = "pure-rust")]
use keyrs_core::output::VirtualDevice;
#[cfg(feature = "pure-rust")]
use keyrs_core::settings::Settings;
#[cfg(feature = "pure-rust")]
use keyrs_core::input::{
    detect_keyboard_type_simple, KeyboardDeviceInfo, KeyboardType,
};
#[cfg(feature = "pure-rust")]
use keyrs_core::transform::engine::TransformEngine;
#[cfg(feature = "pure-rust")]
use keyrs_core::transform::TransformResult;
#[cfg(feature = "pure-rust")]
use keyrs_core::window::WaylandContextProvider;
#[cfg(feature = "pure-rust")]
use keyrs_core::window::WindowContextProvider;
#[cfg(feature = "pure-rust")]
use keyrs_core::Key;

/// Pure Rust Wayland key remapper
#[derive(Parser, Debug)]
#[command(name = "keyrs")]
#[command(author = "keyrs contributors")]
#[command(version = "1.11.3")]
#[command(about = "Pure Rust Wayland key remapper", long_about = None)]
struct Args {
    /// TOML configuration file
    #[arg(short, long, value_name = "CONFIG")]
    config: Option<PathBuf>,

    /// Manually specify devices to remap (can be used multiple times)
    #[arg(short, long, value_name = "DEVICE")]
    devices: Vec<String>,

    /// Watch for hot-plugged devices
    #[arg(short, long)]
    watch: bool,

    /// Enable debug logging
    #[arg(short, long)]
    verbose: bool,

    /// Validate config and exit
    #[arg(long)]
    check_config: bool,

    /// List available keyboard devices
    #[arg(long)]
    list_devices: bool,

    /// Compose modular TOML config directory into a single config file and exit
    #[arg(long, value_name = "DIR")]
    compose_config: Option<PathBuf>,

    /// Output path for --compose-config (default: parent of DIR/config.toml)
    #[arg(long, value_name = "FILE")]
    compose_output: Option<PathBuf>,
}

/// Main application state
#[cfg(feature = "pure-rust")]
struct Application {
    config: Option<Config>,
    args: Args,
    /// Flag to signal event loop to stop
    running: Arc<AtomicBool>,
}

#[cfg(feature = "pure-rust")]
fn resolve_keyboard_type(settings: &Settings, devices: &[KeyboardDeviceInfo]) -> KeyboardType {
    if let Some(override_type) = settings.keyboard_override() {
        if let Some(parsed) = KeyboardType::from_str(override_type) {
            return parsed;
        }
    }

    for device in devices {
        let detected = detect_keyboard_type_simple(device);
        if detected != KeyboardType::Unknown {
            return detected;
        }
    }

    KeyboardType::Unknown
}

#[cfg(feature = "pure-rust")]
fn default_compose_output(dir: &Path) -> PathBuf {
    let base = dir.parent().unwrap_or_else(|| Path::new("."));
    base.join("config.toml")
}

#[cfg(feature = "pure-rust")]
fn merge_table_entries(dst: &mut toml::map::Map<String, Value>, src: toml::map::Map<String, Value>) {
    for (k, v) in src {
        dst.insert(k, v);
    }
}

#[cfg(feature = "pure-rust")]
fn merge_modmap(root: &mut toml::map::Map<String, Value>, src: toml::map::Map<String, Value>) {
    let modmap = root
        .entry("modmap".to_string())
        .or_insert_with(|| Value::Table(toml::map::Map::new()));
    let modmap_tbl = modmap.as_table_mut().expect("modmap must be table");

    for (k, v) in src {
        match (k.as_str(), v) {
            ("default", Value::Table(default_src)) => {
                let default_dst = modmap_tbl
                    .entry("default".to_string())
                    .or_insert_with(|| Value::Table(toml::map::Map::new()));
                let default_tbl = default_dst.as_table_mut().expect("modmap.default must be table");
                merge_table_entries(default_tbl, default_src);
            }
            ("conditionals", Value::Array(src_items)) => {
                let cond_dst = modmap_tbl
                    .entry("conditionals".to_string())
                    .or_insert_with(|| Value::Array(Vec::new()));
                let cond_array = cond_dst.as_array_mut().expect("modmap.conditionals must be array");
                cond_array.extend(src_items);
            }
            (other, value) => {
                modmap_tbl.insert(other.to_string(), value);
            }
        }
    }
}

#[cfg(feature = "pure-rust")]
fn merge_config_fragment(root: &mut toml::map::Map<String, Value>, fragment: toml::map::Map<String, Value>) {
    for (k, v) in fragment {
        match (k.as_str(), v) {
            ("general", Value::Table(src)) | ("timeouts", Value::Table(src)) => {
                let dst = root
                    .entry(k.clone())
                    .or_insert_with(|| Value::Table(toml::map::Map::new()));
                let dst_tbl = dst.as_table_mut().expect("section must be table");
                merge_table_entries(dst_tbl, src);
            }
            ("modmap", Value::Table(src)) => merge_modmap(root, src),
            ("multipurpose", Value::Array(items)) | ("keymap", Value::Array(items)) => {
                let dst = root
                    .entry(k.clone())
                    .or_insert_with(|| Value::Array(Vec::new()));
                let dst_arr = dst.as_array_mut().expect("section must be array");
                dst_arr.extend(items);
            }
            (_, value) => {
                root.insert(k, value);
            }
        }
    }
}

#[cfg(feature = "pure-rust")]
fn compose_config_dir(dir: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
        .collect();
    files.sort();

    if files.is_empty() {
        return Err(format!("No TOML files found in {}", dir.display()).into());
    }

    let mut root = toml::map::Map::new();

    for path in files {
        let content = fs::read_to_string(&path)?;
        let value: Value = toml::from_str(&content)
            .map_err(|e| format!("Failed parsing {}: {}", path.display(), e))?;
        let table = value
            .as_table()
            .ok_or_else(|| format!("Root must be TOML table: {}", path.display()))?
            .clone();
        merge_config_fragment(&mut root, table);
    }

    let rendered = toml::to_string_pretty(&Value::Table(root))?;
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(output, rendered)?;
    Ok(())
}

#[cfg(feature = "pure-rust")]
impl Application {
    /// Create a new application from CLI arguments
    fn new_with_config(
        config_path: PathBuf,
        args: Args,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Load config
        let config = Config::from_toml_path(&config_path)?;

        Ok(Self {
            config: Some(config),
            args,
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Validate configuration
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if config is valid
        if let Some(ref config) = self.config {
            let _transform_config = config.to_transform_config();
            println!("Configuration is valid");
        } else {
            return Err("No configuration loaded".into());
        }
        Ok(())
    }

    /// List available keyboard devices
    #[cfg(feature = "pure-rust")]
    fn list_devices() -> Result<(), Box<dyn std::error::Error>> {
        use keyrs_core::event::EventLoop;

        match EventLoop::list_devices() {
            Ok(devices) => {
                println!("Found {} keyboard device(s):", devices.len());
                for device in &devices {
                    match &device.path {
                        Some(path) => println!("  {}: {} ({})", device.index, device.name, path),
                        None => println!("  {}: {}", device.index, device.name),
                    }
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Error finding keyboard devices: {}", e);
                Err(e.into())
            }
        }
    }

    /// Run the main event loop
    #[cfg(feature = "pure-rust")]
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        use keyrs_core::event::EventLoop;

        if self.args.verbose {
            println!("Starting keyrs pure-rust binary");
            if let Some(ref config_path) = self.args.config {
                println!("Config: {}", config_path.display());
            }
        }

        // Get config
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| Box::<dyn std::error::Error>::from("No configuration loaded"))?;

        // Create transform engine from config
        let transform_config = config.to_transform_config();
        
        // DEBUG: Print loaded keymaps
        println!("\n=== KEYMAP LOADING SUMMARY ===");
        println!("Loaded {} keymaps total", transform_config.keymaps.len());
        for keymap in &transform_config.keymaps {
            println!(
                "  - Keymap '{}': {} mappings, conditional={:?}",
                keymap.name(),
                keymap.mappings().len(),
                keymap.conditional()
            );

            // Show first few mappings for verification
            let sample_size = keymap.mappings().len().min(3);
            if sample_size > 0 {
                println!("    Sample mappings:");
                for (combo, _) in keymap.mappings().iter().take(sample_size) {
                    println!("      - {:?}", combo);
                }
                if keymap.mappings().len() > sample_size {
                    println!("      ... and {} more", keymap.mappings().len() - sample_size);
                }
            }
        }
        println!("=============================\n");
        
        let mut engine = TransformEngine::new(transform_config);

        // Load settings from ~/.config/keyrs/settings.toml
        match Settings::load_default() {
            Ok(settings) => {
                println!("Loaded settings from {:?}", Settings::default_path());
                println!("  Enter2Ent_Cmd = {}", settings.get_bool("Enter2Ent_Cmd"));
                println!("  Caps2Esc_Cmd = {}", settings.get_bool("Caps2Esc_Cmd"));
                println!("  forced_numpad = {}", settings.get_bool("forced_numpad"));
                // Print GenTerms migration flags so runtime condition gating is visible.
                for key in [
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
                ] {
                    println!("  {} = {}", key, settings.get_bool(key));
                }
                engine.set_settings(settings);
            }
            Err(e) => {
                println!("Warning: Could not load settings: {}", e);
            }
        }

        // Set up window context provider for conditional keymaps
        let mut window_provider = WaylandContextProvider::new();
        if let Err(e) = window_provider.connect() {
            if self.args.verbose {
                println!("Warning: Could not connect to window manager: {}", e);
            }
        } else if self.args.verbose {
            println!("Connected to window manager");
        }
        engine.set_window_manager(Some(Box::new(window_provider)));

        // Set up signal handler for graceful shutdown
        #[cfg(feature = "pure-rust")]
        {
            use signal_hook::iterator::Signals;
            let running = self.running.clone();

            // Spawn a thread to handle signals
            std::thread::spawn(move || {
                if let Ok(mut signals) =
                    Signals::new(&[signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM])
                {
                    for signal in &mut signals {
                        match signal {
                            signal_hook::consts::SIGINT | signal_hook::consts::SIGTERM => {
                                println!("\nReceived signal, shutting down gracefully...");
                                running.store(false, Ordering::SeqCst);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            });
        }

        if self.args.verbose {
            println!("Transform engine created");
        }

        // Resolve device filter precedence:
        // CLI --devices > config [devices].only > autodetect.
        let active_device_filter = if !self.args.devices.is_empty() {
            self.args.devices.clone()
        } else {
            config.device_filter.clone()
        };

        // Create event loop with grab (prevents original events from reaching system)
        let mut event_loop = EventLoop::new_with_grab_filtered(&active_device_filter)?;

        if self.args.verbose {
            println!(
                "Event loop created with {} device(s)",
                event_loop.device_count()
            );
            if !active_device_filter.is_empty() {
                println!("Device filter active: {:?}", active_device_filter);
            }
        }

        // Resolve keyboard type with precedence:
        // settings override > auto-detected from active devices > unknown.
        let detection_infos = event_loop.keyboard_detection_infos();
        let settings_for_kb = engine.settings();
        let keyboard_type = resolve_keyboard_type(&settings_for_kb, &detection_infos);
        if keyboard_type == KeyboardType::Unknown {
            engine.clear_keyboard_type();
        } else {
            engine.set_keyboard_type(keyboard_type);
        }
        if self.args.verbose {
            println!("Keyboard type resolved: {}", keyboard_type.as_str());
            if keyboard_type == KeyboardType::Unknown {
                for info in &detection_infos {
                    println!(
                        "  - detect candidate: name='{}' vendor={:?} product={:?} phys={:?}",
                        info.name, info.vendor_id, info.product_id, info.phys
                    );
                }
            }
        }

        // Create virtual uinput device
        let mut output_device = VirtualDevice::new()?;
        output_device.set_throttle_delays(
            config.key_pre_delay_ms.unwrap_or(0),
            config.key_post_delay_ms.unwrap_or(0),
        );

        if self.args.verbose {
            println!("Virtual uinput device created");
            println!(
                "Throttle delays: pre={}ms post={}ms",
                config.key_pre_delay_ms.unwrap_or(0),
                config.key_post_delay_ms.unwrap_or(0)
            );
        }

        // Run main loop
        let result = self.run_main_loop(
            &mut event_loop,
            &mut engine,
            &mut output_device,
            config.diagnostics_key,
            config.emergency_eject_key,
            config.poll_timeout_ms.unwrap_or(100) as i32,
            config.window_update_interval_ms.unwrap_or(500),
            config.idle_sleep_ms.unwrap_or(10),
        );

        // Cleanup: ungrab devices and release keys
        event_loop.ungrab_all();
        let _ = output_device.release_all();
        output_device.close()?;

        result
    }

    /// Run the main event processing loop
    #[cfg(feature = "pure-rust")]
    fn run_main_loop(
        &self,
        event_loop: &mut keyrs_core::event::EventLoop,
        engine: &mut TransformEngine,
        output_device: &mut VirtualDevice,
        diagnostics_key: Option<Key>,
        emergency_eject_key: Option<Key>,
        poll_timeout_ms: i32,
        window_update_interval_ms: u64,
        idle_sleep_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use evdev::EventType;
        use keyrs_core::Action;

        println!("keyrs is running. Press Ctrl+C to exit.");

        // Timestamp for periodic window context updates
        let mut last_window_update = Instant::now();

        while self.running.load(Ordering::SeqCst) {
            // Poll for events with configurable timeout
            match event_loop.poll_for_events_with_device(poll_timeout_ms) {
                Ok(events) => {
                    for event in events {
                        engine.set_device_name(Some(event.device_name.clone()));

                        // Only process key events
                        if event.event.event_type() == EventType::KEY {
                            let key_code = event.event.code() as u16;
                            let value = event.event.value();

                            // Convert evdev value to Action
                            let action = match value {
                                0 => Action::Release,
                                1 => Action::Press,
                                2 => Action::Repeat,
                                _ => continue,
                            };

                            // Process event through transform engine
                            let key = Key::from(key_code);

                            // Emergency eject key: immediate stop for recovery.
                            if Some(key) == emergency_eject_key && action == Action::Press {
                                eprintln!("Emergency eject key pressed. Stopping keyrs.");
                                self.running.store(false, Ordering::SeqCst);
                                continue;
                            }

                            // Diagnostics key: print current context and continue.
                            if Some(key) == diagnostics_key && action == Action::Press {
                                eprintln!("Diagnostics key pressed:");
                                engine.print_window_context();
                                continue;
                            }

                            let result = engine.process_event(key, action);

                            // Log the result if verbose
                            if self.args.verbose {
                                println!("Event: {:?} {:?} -> {:?}", key, action, result);
                            }

                            // Convert to output format and send to uinput device
                            let output = TransformResultOutput::from_transform_result(&result);
                            if let Err(e) = output_device.process_transform_result(&output, action) {
                                eprintln!("Error sending output: {}", e);
                            }
                        }
                    }
                    
                    // Check for multipurpose timeouts after processing events
                    // This handles the case where a key is held longer than the timeout
                    if let Some((hold_key, action)) = engine.check_multipurpose_timeouts() {
                        if self.args.verbose {
                            println!("Multipurpose timeout: {:?} {:?}", hold_key, action);
                        }
                        let result = TransformResult::Remapped(hold_key);
                        let output = TransformResultOutput::from_transform_result(&result);
                        if let Err(e) = output_device.process_transform_result(&output, action) {
                            eprintln!("Error sending output: {}", e);
                        }
                    }
                    
                    // Update window context periodically.
                    if last_window_update.elapsed() >= Duration::from_millis(window_update_interval_ms) {
                        last_window_update = Instant::now();
                        if engine.update_from_window_manager() {
                            if self.args.verbose {
                                println!("Window context updated");
                            }
                            // Always print window info for debugging
                            engine.print_window_context();
                        }
                    }
                }
                Err(_e) => {
                    // No events available, check timeouts anyway (for held keys)
                    if let Some((hold_key, action)) = engine.check_multipurpose_timeouts() {
                        if self.args.verbose {
                            println!("Multipurpose timeout (no events): {:?} {:?}", hold_key, action);
                        }
                        let result = TransformResult::Remapped(hold_key);
                        let output = TransformResultOutput::from_transform_result(&result);
                        if let Err(e) = output_device.process_transform_result(&output, action) {
                            eprintln!("Error sending output: {}", e);
                        }
                    }
                    
                    // Update window context periodically even when no events.
                    if last_window_update.elapsed() >= Duration::from_millis(window_update_interval_ms) {
                        last_window_update = Instant::now();
                        if engine.update_from_window_manager() {
                            if self.args.verbose {
                                println!("Window context updated (no events)");
                            }
                            engine.print_window_context();
                        }
                    }
                    
                    std::thread::sleep(Duration::from_millis(idle_sleep_ms));
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "pure-rust")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Handle list-devices flag (does not require config)
    if args.list_devices {
        return Application::list_devices();
    }

    // Compose modular config and exit (does not require --config).
    if let Some(compose_dir) = args.compose_config.clone() {
        let output = args
            .compose_output
            .clone()
            .unwrap_or_else(|| default_compose_output(&compose_dir));
        compose_config_dir(&compose_dir, &output)?;
        println!("Composed config: {}", output.display());
        return Ok(());
    }

    // Get config path (required for runtime/check mode).
    let config_path = args.config.clone().ok_or_else(|| {
        Box::<dyn std::error::Error>::from("--config is required when not using --list-devices or --compose-config")
    })?;

    // Create application
    let app = Application::new_with_config(config_path, args)?;

    // Handle check-config flag
    if app.args.check_config {
        return app.validate();
    }

    // Run main loop
    app.run()
}

// Stub for when pure-rust feature is not enabled
#[cfg(not(feature = "pure-rust"))]
fn main() {
    eprintln!("Error: keyrs binary requires the 'pure-rust' feature to be enabled.");
    eprintln!("Please build with: cargo build --release --features pure-rust --bin keyrs");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_args_parsing() {
        use std::path::PathBuf;

        // Test basic argument parsing
        let args = Args::parse_from(&["keyrs", "--config", "/tmp/test.toml"]);

        assert_eq!(args.config, Some(PathBuf::from("/tmp/test.toml")));
        assert!(args.devices.is_empty());
        assert!(!args.watch);
        assert!(!args.verbose);
        assert!(!args.check_config);
        assert!(!args.list_devices);
        assert!(args.compose_config.is_none());
        assert!(args.compose_output.is_none());
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_args_with_options() {
        use std::path::PathBuf;

        let args = Args::parse_from(&[
            "keyrs",
            "--config",
            "/tmp/test.toml",
            "--verbose",
            "--watch",
            "--devices",
            "/dev/input/event0",
            "--devices",
            "/dev/input/event1",
        ]);

        assert_eq!(args.config, Some(PathBuf::from("/tmp/test.toml")));
        assert!(args.verbose);
        assert!(args.watch);
        assert_eq!(args.devices.len(), 2);
        assert_eq!(args.devices[0], "/dev/input/event0");
        assert_eq!(args.devices[1], "/dev/input/event1");
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_args_list_devices() {
        let args = Args::parse_from(&["keyrs", "--list-devices"]);

        assert!(args.list_devices);
        assert!(args.compose_config.is_none());
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_args_check_config() {
        use std::path::PathBuf;

        let args = Args::parse_from(&["keyrs", "--config", "/tmp/test.toml", "--check-config"]);

        assert!(args.check_config);
        assert_eq!(args.config, Some(PathBuf::from("/tmp/test.toml")));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_args_compose_config() {
        use std::path::PathBuf;

        let args = Args::parse_from(&[
            "keyrs",
            "--compose-config",
            "./config.d",
            "--compose-output",
            "./generated.toml",
        ]);

        assert_eq!(args.compose_config, Some(PathBuf::from("./config.d")));
        assert_eq!(args.compose_output, Some(PathBuf::from("./generated.toml")));
        assert!(args.config.is_none());
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_compose_config_dir_merges_fragments() {
        let base = std::env::temp_dir().join(format!(
            "keyrs-compose-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let dir = base.join("config.d");
        std::fs::create_dir_all(&dir).expect("create config dir");

        std::fs::write(
            dir.join("000_base.toml"),
            "[general]\nsuspend_key = \"F11\"\n[modmap.default]\nCAPSLOCK = \"CAPSLOCK\"\n",
        )
        .expect("write base");
        std::fs::write(
            dir.join("100_terminal.toml"),
            "[[keymap]]\nname = \"k1\"\n[keymap.mappings]\n\"Super-c\" = \"Ctrl-c\"\n",
        )
        .expect("write fragment");

        let out = base.join("config.toml");
        compose_config_dir(&dir, &out).expect("compose");

        let rendered = std::fs::read_to_string(&out).expect("read output");
        assert!(rendered.contains("suspend_key"));
        assert!(rendered.contains("[[keymap]]"));
        assert!(rendered.contains("CAPSLOCK"));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_default_compose_output_uses_parent_directory() {
        let dir = PathBuf::from("./config.d.example");
        let out = default_compose_output(&dir);
        assert_eq!(out, PathBuf::from("./config.toml"));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_compose_config_dir_orders_by_filename_and_merges_modmap_default() {
        let base = std::env::temp_dir().join(format!(
            "keyrs-compose-order-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let dir = base.join("config.d");
        std::fs::create_dir_all(&dir).expect("create config dir");

        // Intentionally write files out of lexical order to verify sorting.
        std::fs::write(
            dir.join("200_keymaps.toml"),
            "[[keymap]]\nname = \"later\"\n[keymap.mappings]\n\"Super-v\" = \"Ctrl-v\"\n",
        )
        .expect("write keymaps");
        std::fs::write(
            dir.join("010_base.toml"),
            "[modmap.default]\nCAPSLOCK = \"RIGHT_CTRL\"\n",
        )
        .expect("write base");
        std::fs::write(
            dir.join("020_modmap_override.toml"),
            "[modmap.default]\nCAPSLOCK = \"CAPSLOCK\"\n",
        )
        .expect("write override");

        let out = base.join("config.toml");
        compose_config_dir(&dir, &out).expect("compose");

        let rendered = std::fs::read_to_string(&out).expect("read output");
        // Later fragment should override earlier modmap.default key.
        assert!(rendered.contains("CAPSLOCK = \"CAPSLOCK\""));
        // Keymap must be present after merge.
        assert!(rendered.contains("name = \"later\""));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_resolve_keyboard_type_uses_override_first() {
        let settings = Settings::from_toml(
            r#"
            [keyboard]
            override_type = "Apple"
            "#,
        )
        .unwrap();

        let device_infos = vec![KeyboardDeviceInfo::new("IBM Model M")];
        let kb_type = resolve_keyboard_type(&settings, &device_infos);
        assert_eq!(kb_type, KeyboardType::Mac);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_resolve_keyboard_type_detects_from_device_name() {
        let settings = Settings::new();
        let device_infos = vec![KeyboardDeviceInfo::new("Lenovo ThinkPad Compact USB Keyboard")];
        let kb_type = resolve_keyboard_type(&settings, &device_infos);
        assert_eq!(kb_type, KeyboardType::IBM);
    }
}

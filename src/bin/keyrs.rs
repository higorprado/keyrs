// Xwaykeyz Pure Rust CLI
// Standalone binary for Wayland key remapping without Python dependencies

#![cfg_attr(feature = "pure-rust", allow(dead_code))]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "pure-rust")]
use clap::Parser;

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
    ) -> Result<(), Box<dyn std::error::Error>> {
        use evdev::EventType;
        use keyrs_core::Action;

        println!("keyrs is running. Press Ctrl+C to exit.");

        // Counter for periodic window context updates
        let mut window_update_counter: u32 = 0;

        while self.running.load(Ordering::SeqCst) {
            // Poll for events with 100ms timeout
            match event_loop.poll_for_events_with_device(100) {
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
                    
                    // Update window context periodically (every ~500ms)
                    window_update_counter += 1;
                    if window_update_counter >= 5 {
                        window_update_counter = 0;
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
                    
                    // Update window context periodically even when no events
                    window_update_counter += 1;
                    if window_update_counter >= 5 {
                        window_update_counter = 0;
                        if engine.update_from_window_manager() {
                            if self.args.verbose {
                                println!("Window context updated (no events)");
                            }
                            engine.print_window_context();
                        }
                    }
                    
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "pure-rust")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Handle list-devices flag (doesn't require config)
    if args.list_devices {
        return Application::list_devices();
    }

    // Get config path (required for other operations)
    let config_path = args.config.clone().ok_or_else(|| {
        Box::<dyn std::error::Error>::from("--config is required when not using --list-devices")
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

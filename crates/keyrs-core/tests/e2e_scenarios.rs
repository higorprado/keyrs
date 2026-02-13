// Keyrs End-to-End Test Scenarios
//
// These tests simulate real-world usage scenarios for validation.
// They test complete user workflows without requiring actual hardware.
//
// Run with: cargo test --features pure-rust --test e2e_scenarios

#[cfg(feature = "pure-rust")]
mod e2e_tests {
    use std::collections::HashMap;
    use std::time::Duration;
    use keyrs_core::input::{KeyboardType, keyboard_type_matches};
    use keyrs_core::mapping::{Keymap, KeymapValue, Modmap, MultiModmap, MultipurposeManager};
    use keyrs_core::settings::Settings;
    use keyrs_core::transform::engine::{TransformConfig, TransformEngine, TransformResult, WindowContext};
    use keyrs_core::window::{WindowCondition, WindowInfo};
    use keyrs_core::{Action, Combo, Key, Modifier};

    // =========================================================================
    // Test Helpers
    // =========================================================================

    /// Simulate a key press and release sequence
    fn tap_key(engine: &mut TransformEngine, key: Key) -> Vec<TransformResult> {
        vec![
            engine.process_event(key, Action::Press),
            engine.process_event(key, Action::Release),
        ]
    }

    /// Simulate holding a key while pressing another
    fn hold_and_press(
        engine: &mut TransformEngine,
        hold_key: Key,
        press_key: Key,
    ) -> Vec<TransformResult> {
        vec![
            engine.process_event(hold_key, Action::Press),
            engine.process_event(press_key, Action::Press),
            engine.process_event(press_key, Action::Release),
            engine.process_event(hold_key, Action::Release),
        ]
    }

    /// Create a Caps2Esc configuration (the most common use case)
    fn create_caps2esc_config() -> TransformConfig {
        let mut modmap_mappings = HashMap::new();
        modmap_mappings.insert(Key::from(58), Key::from(1)); // CAPSLOCK -> ESCAPE

        TransformConfig {
            modmaps: vec![Modmap::new("caps2esc", modmap_mappings)],
            multimodmaps: vec![],
            keymaps: vec![],
            suspend_key: None,
            multipurpose_timeout: Some(200),
            suspend_timeout: Some(1000),
        }
    }

    /// Create Enter2Cmd configuration
    fn create_enter2cmd_config() -> TransformConfig {
        let mut config = TransformConfig::default();
        // Add multipurpose entry in the engine after creation
        config.multipurpose_timeout = Some(200);
        config
    }

    /// Create a comprehensive IDE-like configuration
    fn create_ide_config() -> TransformConfig {
        // Modmaps
        let mut modmap_mappings = HashMap::new();
        modmap_mappings.insert(Key::from(58), Key::from(29)); // CAPSLOCK -> LEFT_CTRL

        let modmap = Modmap::new("ide_modmaps", modmap_mappings);

        // Keymaps with common IDE shortcuts
        let mut keymap = Keymap::new("ide_shortcuts");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let shift = Modifier::from_alias("Shift").unwrap();
        let alt = Modifier::from_alias("Alt").unwrap();

        // Ctrl+P -> Quick Open (simulated with F1)
        let combo = Combo::from_single(ctrl.clone(), Key::from(25)); // Ctrl+P
        keymap.insert(combo, KeymapValue::Key(Key::from(59))); // -> F1

        // Ctrl+Shift+F -> Find in Files (simulated with F2)
        let combo = Combo::new(vec![ctrl, shift], Key::from(33)); // Ctrl+Shift+F
        keymap.insert(combo, KeymapValue::Key(Key::from(60))); // -> F2

        // Alt+Enter -> Quick Fix (simulated with F3)
        let combo = Combo::from_single(alt, Key::from(28)); // Alt+Enter
        keymap.insert(combo, KeymapValue::Key(Key::from(61))); // -> F3

        TransformConfig {
            modmaps: vec![modmap],
            multimodmaps: vec![],
            keymaps: vec![keymap],
            suspend_key: None,
            multipurpose_timeout: Some(200),
            suspend_timeout: Some(1000),
        }
    }

    // =========================================================================
    // E2E Scenario 1: Caps2Esc - The Essential Modmap
    // =========================================================================

    #[test]
    fn e2e_caps2esc_tap_for_escape() {
        // Scenario: User taps Caps Lock expecting Escape
        let config = create_caps2esc_config();
        let mut engine = TransformEngine::new(config);

        // Tap Caps Lock
        let results = tap_key(&mut engine, Key::from(58));

        // Both press and release should be remapped to Escape
        assert_eq!(results[0], TransformResult::Remapped(Key::from(1)));
        assert_eq!(results[1], TransformResult::Remapped(Key::from(1)));
    }

    #[test]
    fn e2e_caps2esc_does_not_affect_other_keys() {
        // Scenario: Ensure Caps2Esc doesn't interfere with other keys
        let config = create_caps2esc_config();
        let mut engine = TransformEngine::new(config);

        // Type "hello" (h=35, e=18, l=38, o=24)
        let keys = vec![
            Key::from(35),
            Key::from(18),
            Key::from(38),
            Key::from(38),
            Key::from(24),
        ];

        for key in keys {
            let result = engine.process_event(key, Action::Press);
            // All should passthrough unchanged
            assert!(
                matches!(result, TransformResult::Passthrough(k) if k == key),
                "Key {:?} should passthrough but got {:?}",
                key,
                result
            );
        }
    }

    // =========================================================================
    // E2E Scenario 2: Enter2Cmd - Multipurpose Modmap
    // =========================================================================

    #[test]
    fn e2e_enter2cmd_tap_for_enter() {
        // Scenario: User taps Enter quickly, expecting normal Enter behavior
        let config = create_enter2cmd_config();
        let mut engine = TransformEngine::new(config);

        // Add multipurpose mapping: Enter -> Enter (tap), RCtrl (hold)
        engine.add_multipurpose(Key::from(28), Key::from(28), Key::from(97));

        // Press Enter
        let result = engine.process_event(Key::from(28), Action::Press);
        assert_eq!(result, TransformResult::Suppress, "Should suppress initial press");

        // Release quickly (tap)
        let result = engine.process_event(Key::from(28), Action::Release);
        assert_eq!(
            result,
            TransformResult::Remapped(Key::from(28)),
            "Should output Enter on tap"
        );
    }

    #[test]
    fn e2e_enter2cmd_interrupt_for_ctrl() {
        // Scenario: User holds Enter and presses another key, expecting Ctrl combo
        let config = create_enter2cmd_config();
        let mut engine = TransformEngine::new(config);

        // Add multipurpose mapping: Enter -> Enter (tap), RCtrl (hold)
        engine.add_multipurpose(Key::from(28), Key::from(28), Key::from(97));

        // Press Enter (starts multipurpose sequence)
        let _ = engine.process_event(Key::from(28), Action::Press);

        // Interrupt with 'a' key (should trigger hold mode)
        let result = engine.process_event(Key::from(30), Action::Press); // 'a'

        // Should be processing the interrupting key normally
        // The hold key (RCtrl) was output internally
        assert!(
            matches!(result, TransformResult::Passthrough(_) | TransformResult::ComboKey(_)),
            "Should process interrupting key, got {:?}",
            result
        );
    }

    // =========================================================================
    // E2E Scenario 3: IDE Shortcuts
    // =========================================================================

    #[test]
    fn e2e_ide_quick_open() {
        // Scenario: Developer presses Ctrl+P for Quick Open
        let config = create_ide_config();
        let mut engine = TransformEngine::new(config);

        // Press Ctrl
        let _ = engine.process_event(Key::from(29), Action::Press);

        // Press P
        let result = engine.process_event(Key::from(25), Action::Press);

        // Should trigger Quick Open (F1)
        assert_eq!(
            result,
            TransformResult::ComboKey(Key::from(59)),
            "Ctrl+P should trigger Quick Open"
        );
    }

    #[test]
    fn e2e_ide_find_in_files() {
        // Scenario: Developer presses Ctrl+Shift+F for Find in Files
        let config = create_ide_config();
        let mut engine = TransformEngine::new(config);

        // Press Ctrl
        let _ = engine.process_event(Key::from(29), Action::Press);
        // Press Shift
        let _ = engine.process_event(Key::from(42), Action::Press);

        // Press F
        let result = engine.process_event(Key::from(33), Action::Press);

        // Should trigger Find in Files (F2)
        assert_eq!(
            result,
            TransformResult::ComboKey(Key::from(60)),
            "Ctrl+Shift+F should trigger Find in Files"
        );
    }

    // =========================================================================
    // E2E Scenario 4: Window Context Conditions
    // =========================================================================

    #[test]
    fn e2e_window_condition_matches_firefox() {
        // Scenario: Firefox-specific modmap should only match Firefox
        let condition = WindowCondition::parse("wm_class =~ 'Firefox'").unwrap();

        let firefox_info = WindowInfo::with_details(
            Some("org.mozilla.firefox".to_string()),
            Some("GitHub - Firefox".to_string()),
        );

        let chrome_info = WindowInfo::with_details(
            Some("google-chrome".to_string()),
            Some("GitHub - Chrome".to_string()),
        );

        assert!(
            firefox_info.matches_condition(&condition),
            "Should match Firefox"
        );
        assert!(
            !chrome_info.matches_condition(&condition),
            "Should not match Chrome"
        );
    }

    #[test]
    fn e2e_window_condition_case_insensitive() {
        // Scenario: Window matching should be case-insensitive
        let condition = WindowCondition::parse("wm_class =~ 'firefox'").unwrap();

        let uppercase = WindowInfo::with_details(Some("FIREFOX".to_string()), None);
        let mixed_case = WindowInfo::with_details(Some("Firefox".to_string()), None);
        let lowercase = WindowInfo::with_details(Some("firefox".to_string()), None);

        assert!(uppercase.matches_condition(&condition), "Should match uppercase");
        assert!(mixed_case.matches_condition(&condition), "Should match mixed case");
        assert!(lowercase.matches_condition(&condition), "Should match lowercase");
    }

    // =========================================================================
    // E2E Scenario 5: Keyboard Type Detection
    // =========================================================================

    #[test]
    fn e2e_keyboard_type_matches_single() {
        // Scenario: IBM keyboard modmap should match IBM keyboards
        assert!(keyboard_type_matches(KeyboardType::IBM, "IBM"));
        assert!(!keyboard_type_matches(KeyboardType::IBM, "Mac"));
    }

    #[test]
    fn e2e_keyboard_type_matches_list() {
        // Scenario: Universal modmap should match multiple keyboard types
        assert!(keyboard_type_matches(KeyboardType::IBM, "IBM, Chromebook, Windows"));
        assert!(keyboard_type_matches(KeyboardType::Chromebook, "IBM, Chromebook, Windows"));
        assert!(keyboard_type_matches(KeyboardType::Windows, "IBM, Chromebook, Windows"));
        assert!(!keyboard_type_matches(KeyboardType::Mac, "IBM, Chromebook, Windows"));
    }

    #[test]
    fn e2e_keyboard_type_case_insensitive() {
        // Scenario: Keyboard type matching should be case-insensitive
        assert!(keyboard_type_matches(KeyboardType::IBM, "ibm"));
        assert!(keyboard_type_matches(KeyboardType::Chromebook, "chromebook"));
        assert!(keyboard_type_matches(KeyboardType::Mac, "MAC"));
    }

    // =========================================================================
    // E2E Scenario 6: Complex Workflows
    // =========================================================================

    #[test]
    fn e2e_complex_typing_with_modifiers() {
        // Scenario: User types with Caps Lock (remapped to Ctrl) held
        let config = create_ide_config();
        let mut engine = TransformEngine::new(config);

        // Hold Caps Lock (remapped to Ctrl)
        let result = engine.process_event(Key::from(58), Action::Press);
        assert_eq!(result, TransformResult::Remapped(Key::from(29)));

        // Press C (should be Ctrl+C)
        let result = engine.process_event(Key::from(46), Action::Press);
        assert!(
            matches!(result, TransformResult::ComboKey(_) | TransformResult::Passthrough(_)),
            "Ctrl+C should work"
        );

        // Release C
        let _ = engine.process_event(Key::from(46), Action::Release);

        // Release Caps Lock
        let result = engine.process_event(Key::from(58), Action::Release);
        assert_eq!(result, TransformResult::Remapped(Key::from(29)));
    }

    #[test]
    fn e2e_suspend_and_resume() {
        // Scenario: User suspends key remapping, does something, then resumes
        let config = create_caps2esc_config();
        let mut engine = TransformEngine::new(config);

        // Normal behavior - Caps Lock remaps to Escape
        let result = engine.process_event(Key::from(58), Action::Press);
        assert_eq!(result, TransformResult::Remapped(Key::from(1)));

        // Suspend
        engine.suspend();

        // While suspended, all keys are suppressed (keyrs disables itself)
        let result = engine.process_event(Key::from(58), Action::Press);
        assert_eq!(result, TransformResult::Suppress, "While suspended, keys should be suppressed");

        // Resume (by pressing any key - suspend mode exits on any key in this implementation)
        // or we can explicitly resume
        engine.resume();

        // Back to normal behavior
        let result = engine.process_event(Key::from(58), Action::Press);
        assert_eq!(result, TransformResult::Remapped(Key::from(1)));
    }

    #[test]
    fn e2e_keystore_state_consistency() {
        // Scenario: Ensure keystore state is consistent after various operations
        let config = create_ide_config();
        let mut engine = TransformEngine::new(config);

        // Press multiple keys
        let _ = engine.process_event(Key::from(29), Action::Press); // Ctrl
        let _ = engine.process_event(Key::from(42), Action::Press); // Shift
        let _ = engine.process_event(Key::from(30), Action::Press); // A

        // Verify all are tracked
        assert_eq!(engine.keystore().read().len(), 3);
        assert!(engine.keystore().read().get(29).is_some());
        assert!(engine.keystore().read().get(42).is_some());
        assert!(engine.keystore().read().get(30).is_some());

        // Release in different order
        let _ = engine.process_event(Key::from(30), Action::Release);
        assert!(!engine.keystore().read().get(30).unwrap().key_is_pressed());

        let _ = engine.process_event(Key::from(29), Action::Release);
        assert!(!engine.keystore().read().get(29).unwrap().key_is_pressed());

        let _ = engine.process_event(Key::from(42), Action::Release);
        assert!(!engine.keystore().read().get(42).unwrap().key_is_pressed());

        // All released but still tracked
        assert_eq!(engine.keystore().read().len(), 3);
    }

    // =========================================================================
    // E2E Scenario 7: Error Recovery
    // =========================================================================

    #[test]
    fn e2e_clear_resets_all_state() {
        // Scenario: Clear should reset everything to initial state
        let config = create_ide_config();
        let mut engine = TransformEngine::new(config);

        // Build up some state
        let _ = engine.process_event(Key::from(29), Action::Press);
        let _ = engine.process_event(Key::from(42), Action::Press);
        let _ = engine.process_event(Key::from(30), Action::Press);

        // Suspend the engine
        engine.suspend();

        // Set mark via hint processing
        // (mark is set via escape_next flag during processing)

        // Clear everything
        engine.clear();

        // Verify reset
        assert_eq!(engine.keystore().read().len(), 0);
        assert_eq!(engine.get_mark(), None);
    }

    #[test]
    fn e2e_multipurpose_timeout_handling() {
        // Scenario: Multipurpose key should transition to hold after timeout
        let mut manager = MultipurposeManager::with_timeout(10); // 10ms timeout

        let mut modmap_mappings = HashMap::new();
        modmap_mappings.insert(Key::from(58), (Key::from(1), Key::from(97))); // CAPS -> ESC/RCtrl
        let modmap = MultiModmap::new("test", modmap_mappings);
        manager.add_modmap(modmap);

        // Start multipurpose
        assert!(manager.start(Key::from(58)));

        // Should be pending
        assert!(manager.is_pending_state());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(50));

        // Check timeout
        let result = manager.check_timeout();
        assert!(result.is_some(), "Should detect timeout");
        assert_eq!(result.unwrap(), Key::from(97), "Should return RCtrl");
    }

    // =========================================================================
    // E2E Scenario: Settings Integration
    // =========================================================================

    #[test]
    fn e2e_settings_based_conditions() {
        use keyrs_core::settings::Settings;
        
        // Scenario: Settings control whether multipurpose keys are active
        let mut settings = Settings::new();
        settings.set_bool("Caps2Esc_Cmd", true);
        settings.set_bool("Enter2Ent_Cmd", false);
        
        // Verify settings work
        assert_eq!(settings.get_bool("Caps2Esc_Cmd"), true);
        assert_eq!(settings.get_bool("Enter2Ent_Cmd"), false);
        
        // Test condition evaluation
        assert_eq!(settings.evaluate_condition("settings.Caps2Esc_Cmd"), true);
        assert_eq!(settings.evaluate_condition("settings.Enter2Ent_Cmd"), false);
        assert_eq!(settings.evaluate_condition("not settings.Enter2Ent_Cmd"), true);
        
        // Test WindowContext with settings
        let mut context = WindowContext::default();
        context.set_settings(settings);
        
        // Test that conditions work in context
        assert_eq!(context.matches_condition("settings.Caps2Esc_Cmd"), true);
        assert_eq!(context.matches_condition("settings.Enter2Ent_Cmd"), false);
        assert_eq!(context.matches_condition("not settings.Enter2Ent_Cmd"), true);
    }

    #[test]
    fn e2e_transform_engine_with_settings() {
        use keyrs_core::settings::Settings;
        
        // Create engine with default config
        let config = TransformConfig::default();
        let mut engine = TransformEngine::new(config);
        
        // Set custom settings
        let mut settings = Settings::new();
        settings.set_bool("Enter2Ent_Cmd", true);
        engine.set_settings(settings);
        
        // Verify settings are accessible
        assert_eq!(engine.get_setting("Enter2Ent_Cmd"), true);
        assert_eq!(engine.get_setting("Caps2Esc_Cmd"), false); // default
        
        // Get settings reference
        let retrieved_settings = engine.settings();
        assert_eq!(retrieved_settings.get_bool("Enter2Ent_Cmd"), true);
    }
}

// =========================================================================
// Benchmark-style Tests (not actual benchmarks, but performance checks)
// =========================================================================

#[cfg(feature = "pure-rust")]
mod performance_tests {
    use std::collections::HashMap;
    use std::time::Instant;
    use keyrs_core::mapping::Modmap;
    use keyrs_core::transform::engine::{TransformConfig, TransformEngine};
    use keyrs_core::{Action, Key};

    #[test]
    fn perf_single_key_latency() {
        // Ensure single key processing is under 1ms (well under for responsiveness)
        let config = TransformConfig::default();
        let mut engine = TransformEngine::new(config);

        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = engine.process_event(Key::from(30), Action::Press);
            let _ = engine.process_event(Key::from(30), Action::Release);
        }

        let elapsed = start.elapsed();
        let avg_per_key = elapsed / (iterations * 2);

        println!(
            "Single key latency: {:?} (avg per key, {} iterations)",
            avg_per_key, iterations
        );

        // Should be well under 1ms
        assert!(
            avg_per_key < Duration::from_micros(100),
            "Single key processing too slow: {:?}",
            avg_per_key
        );
    }

    #[test]
    fn perf_modmap_lookup() {
        // Test modmap lookup performance with many mappings
        let mut modmap_mappings = HashMap::new();
        
        // Add 100 modmap entries
        for i in 0..100 {
            modmap_mappings.insert(Key::from(i), Key::from(i + 100));
        }

        let config = TransformConfig {
            modmaps: vec![Modmap::new("large", modmap_mappings)],
            multimodmaps: vec![],
            keymaps: vec![],
            suspend_key: None,
            multipurpose_timeout: Some(200),
            suspend_timeout: Some(1000),
        };

        let mut engine = TransformEngine::new(config);

        let iterations = 1000;
        let start = Instant::now();

        for i in 0..iterations {
            let key = Key::from((i % 100) as u16);
            let _ = engine.process_event(key, Action::Press);
        }

        let elapsed = start.elapsed();
        let avg = elapsed / iterations;

        println!("Modmap lookup latency: {:?} (avg, {} iterations)", avg, iterations);

        // Should still be under 100 microseconds even with 100 entries
        assert!(
            avg < Duration::from_micros(100),
            "Modmap lookup too slow: {:?}",
            avg
        );
    }

    use std::time::Duration;
}

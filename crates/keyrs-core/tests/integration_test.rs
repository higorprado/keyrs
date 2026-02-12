// Xwaykeyz Pure Rust Integration Tests
//
// These tests verify the complete pure Rust pipeline:
// evdev -> EventLoop -> TransformEngine -> output
//
// Run with: cargo test --features pure-rust --test integration_test

#[cfg(feature = "pure-rust")]
mod tests {
    use std::collections::HashMap;
    use keyrs_core::mapping::{Keymap, KeymapValue, Modmap};
    use keyrs_core::transform::engine::{TransformConfig, TransformEngine, TransformResult};
    use keyrs_core::{Action, Combo, Key, Modifier};

    // Helper function to create a sample transform config
    fn create_sample_config() -> TransformConfig {
        // Create modmap: CAPSLOCK -> LEFT_CTRL
        let mut modmap_mappings = HashMap::new();
        modmap_mappings.insert(Key::from(58), Key::from(29)); // CAPSLOCK -> LEFT_CTRL

        let modmap = Modmap::new("default", modmap_mappings);

        // Create keymap: Ctrl+A -> B
        let mut keymap = Keymap::new("default");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::from_single(ctrl, Key::from(30)); // Ctrl+A
        keymap.insert(combo, KeymapValue::Key(Key::from(31))); // -> B

        TransformConfig {
            modmaps: vec![modmap],
            multimodmaps: vec![],
            keymaps: vec![keymap],
            suspend_key: None,
            multipurpose_timeout: Some(500),
            suspend_timeout: Some(1000),
        }
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_config_creation() {
        let config = create_sample_config();
        assert_eq!(config.modmaps.len(), 1);
        assert_eq!(config.keymaps.len(), 1);
        assert!(config.suspend_key.is_none());
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_creation() {
        let config = create_sample_config();
        let engine = TransformEngine::new(config);
        assert_eq!(engine.keystore().read().len(), 0);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_passthrough() {
        let config = TransformConfig::default();
        let mut engine = TransformEngine::new(config);

        // Process a simple key press (A)
        let result = engine.process_event(Key::from(30), Action::Press);

        // Should passthrough since no modmap or keymap
        assert_eq!(result, TransformResult::Passthrough(Key::from(30)));

        // Check keystore has the key
        assert_eq!(engine.keystore().read().len(), 1);
        let store = engine.keystore().read();
        let keystate = store.get(30);
        assert!(keystate.is_some());
        assert_eq!(keystate.unwrap().inkey.code(), 30);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_modmap() {
        let config = create_sample_config();
        let mut engine = TransformEngine::new(config);

        // Process CAPSLOCK press (should be remapped to LEFT_CTRL)
        let result = engine.process_event(Key::from(58), Action::Press);

        // Should be remapped
        assert_eq!(result, TransformResult::Remapped(Key::from(29)));

        // Check keystore has the key
        assert_eq!(engine.keystore().read().len(), 1);
        let store = engine.keystore().read();
        let keystore = store.get(58);
        assert!(keystore.is_some());
        assert_eq!(keystore.unwrap().inkey.code(), 58);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_combo() {
        let config = create_sample_config();
        let mut engine = TransformEngine::new(config);

        // First press Ctrl
        let _ = engine.process_event(Key::from(29), Action::Press);

        // Then press A with Ctrl held
        let result = engine.process_event(Key::from(30), Action::Press);

        // Should match the combo and output B
        assert_eq!(result, TransformResult::ComboKey(Key::from(31)));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_repeat_cache() {
        let config = create_sample_config();
        let mut engine = TransformEngine::new(config);

        // Press A
        let result1 = engine.process_event(Key::from(30), Action::Press);
        assert!(matches!(result1, TransformResult::Passthrough(_)));

        // Repeat A (should use cache)
        let result2 = engine.process_event(Key::from(30), Action::Repeat);
        assert_eq!(result2, result1);

        // Press another key (invalidates cache)
        let _ = engine.process_event(Key::from(31), Action::Press);

        // Repeat A again (should re-process)
        let result3 = engine.process_event(Key::from(30), Action::Repeat);
        assert!(matches!(result3, TransformResult::Passthrough(_)));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_clear() {
        let config = create_sample_config();
        let mut engine = TransformEngine::new(config);

        // Add some keys
        let _ = engine.process_event(Key::from(30), Action::Press);
        let _ = engine.process_event(Key::from(31), Action::Press);
        assert_eq!(engine.keystore().read().len(), 2);

        // Clear
        engine.clear();
        assert_eq!(engine.keystore().read().len(), 0);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_keystore_modifier_snapshot() {
        let config = create_sample_config();
        let mut engine = TransformEngine::new(config);

        // Press some modifiers
        let _ = engine.process_event(Key::from(29), Action::Press); // LEFT_CTRL
        let _ = engine.process_event(Key::from(56), Action::Press); // LEFT_ALT
        let _ = engine.process_event(Key::from(30), Action::Press); // A (not a modifier)

        // Get modifier snapshot
        let snapshot = engine.keystore().read().get_modifier_snapshot();

        // Should contain only modifier codes (29, 56), sorted
        assert_eq!(snapshot.as_slice(), &[29, 56]);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_keystore_pressed_mods() {
        let config = create_sample_config();
        let mut engine = TransformEngine::new(config);

        // Press some modifiers
        let _ = engine.process_event(Key::from(29), Action::Press); // LEFT_CTRL
        let _ = engine.process_event(Key::from(56), Action::Press); // LEFT_ALT
        let _ = engine.process_event(Key::from(30), Action::Press); // A (not a modifier)

        // Get pressed modifier keys
        let mods = engine.keystore().read().get_pressed_mods_keys();

        // Should get 2 modifier keys
        assert_eq!(mods.len(), 2);
        assert!(mods.contains(&Key::from(29))); // LEFT_CTRL
        assert!(mods.contains(&Key::from(56))); // LEFT_ALT
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_key_sequence() {
        let config = create_sample_config();
        let mut engine = TransformEngine::new(config);

        // Simulate typing "hello" (h=35, e=18, l=38, l=38, o=24)
        let keys = vec![
            (Key::from(35), Action::Press),
            (Key::from(35), Action::Release),
            (Key::from(18), Action::Press),
            (Key::from(18), Action::Release),
            (Key::from(38), Action::Press),
            (Key::from(38), Action::Release),
            (Key::from(38), Action::Press),
            (Key::from(38), Action::Release),
            (Key::from(24), Action::Press),
            (Key::from(24), Action::Release),
        ];

        for (key, action) in keys {
            let result = engine.process_event(key, action);
            // All should passthrough since no modmap or keymap matches
            assert!(matches!(
                result,
                TransformResult::Passthrough(_) | TransformResult::Remapped(_)
            ));
        }

        // All keys should be in keystore (at various states)
        // We have 4 unique keys (h=35, e=18, l=38, o=24) but l=38 is pressed twice
        // So we should have at least 4 entries
        assert!(engine.keystore().read().len() >= 4);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_result_equality() {
        // Test TransformResult equality
        let r1 = TransformResult::Passthrough(Key::from(30));
        let r2 = TransformResult::Passthrough(Key::from(30));
        let r3 = TransformResult::Passthrough(Key::from(31));
        let r4 = TransformResult::Remapped(Key::from(30));

        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
        assert_ne!(r1, r4);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_combo_matching_with_multiple_modifiers() {
        // Create config with multi-modifier combo
        let modmap_mappings = HashMap::new();
        let modmap = Modmap::new("default", modmap_mappings);

        let mut keymap = Keymap::new("default");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let shift = Modifier::from_alias("Shift").unwrap();
        let combo = Combo::new(vec![ctrl, shift], Key::from(30)); // Ctrl+Shift+A
        keymap.insert(combo, KeymapValue::Key(Key::from(31))); // -> B

        let config = TransformConfig {
            modmaps: vec![modmap],
            multimodmaps: vec![],
            keymaps: vec![keymap],
            suspend_key: None,
            multipurpose_timeout: Some(500),
            suspend_timeout: Some(1000),
        };

        let mut engine = TransformEngine::new(config);

        // Press Ctrl
        let _ = engine.process_event(Key::from(29), Action::Press);
        // Press Shift
        let _ = engine.process_event(Key::from(42), Action::Press);
        // Press A
        let result = engine.process_event(Key::from(30), Action::Press);

        // Should match the combo and output B
        assert_eq!(result, TransformResult::ComboKey(Key::from(31)));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_multiple_modmaps() {
        // Create config with multiple modmaps
        let mut modmap1_mappings = HashMap::new();
        modmap1_mappings.insert(Key::from(58), Key::from(29)); // CAPSLOCK -> LEFT_CTRL
        let modmap1 = Modmap::new("default", modmap1_mappings);

        let mut modmap2_mappings = HashMap::new();
        modmap2_mappings.insert(Key::from(42), Key::from(29)); // LEFT_SHIFT -> LEFT_CTRL
        let modmap2 = Modmap::new("conditional", modmap2_mappings);

        let config = TransformConfig {
            modmaps: vec![modmap1, modmap2],
            multimodmaps: vec![],
            keymaps: vec![],
            suspend_key: None,
            multipurpose_timeout: Some(500),
            suspend_timeout: Some(1000),
        };

        let mut engine = TransformEngine::new(config);

        // Test first modmap (CAPSLOCK -> LEFT_CTRL)
        let result1 = engine.process_event(Key::from(58), Action::Press);
        assert_eq!(result1, TransformResult::Remapped(Key::from(29)));

        // Clear for next test
        engine.clear();

        // Test second modmap (LEFT_SHIFT -> LEFT_CTRL)
        // Note: Current implementation only checks first modmap
        // This test documents current behavior
        let result2 = engine.process_event(Key::from(42), Action::Press);
        // Currently won't match since it only checks first modmap
        assert!(matches!(result2, TransformResult::Passthrough(_)));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_conditional_keymap_matches_device_name_and_lock_state() {
        let modmap = Modmap::new("default", HashMap::new());

        let mut mappings = HashMap::new();
        mappings.insert(
            Combo::new(Vec::<Modifier>::new(), Key::from(30)), // A
            KeymapValue::Key(Key::from(31)),                   // -> B
        );
        let keymap = Keymap::with_conditional(
            "device_conditional",
            mappings,
            "device_name =~ 'Telink' and not numlock and capslock == false".to_string(),
        );

        let config = TransformConfig {
            modmaps: vec![modmap],
            multimodmaps: vec![],
            keymaps: vec![keymap],
            suspend_key: None,
            multipurpose_timeout: Some(500),
            suspend_timeout: Some(1000),
        };

        let mut engine = TransformEngine::new(config);
        engine.set_device_name(Some("Telink Wireless Gaming Keyboard".to_string()));
        engine.set_lock_states(false, false);

        let result = engine.process_event(Key::from(30), Action::Press);
        assert_eq!(result, TransformResult::ComboKey(Key::from(31)));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_conditional_keymap_blocked_when_condition_false() {
        let modmap = Modmap::new("default", HashMap::new());

        let mut mappings = HashMap::new();
        mappings.insert(
            Combo::new(Vec::<Modifier>::new(), Key::from(30)), // A
            KeymapValue::Key(Key::from(31)),                   // -> B
        );
        let keymap = Keymap::with_conditional(
            "device_conditional",
            mappings,
            "device_name =~ 'Telink' and not numlock".to_string(),
        );

        let config = TransformConfig {
            modmaps: vec![modmap],
            multimodmaps: vec![],
            keymaps: vec![keymap],
            suspend_key: None,
            multipurpose_timeout: Some(500),
            suspend_timeout: Some(1000),
        };

        let mut engine = TransformEngine::new(config);
        engine.set_device_name(Some("Some Other Keyboard".to_string()));
        engine.set_lock_states(false, false);

        let result = engine.process_event(Key::from(30), Action::Press);
        assert_eq!(result, TransformResult::Passthrough(Key::from(30)));
    }
}

// Tests that require actual keyboard devices (may be skipped in CI)
// These tests are marked as ignored by default to prevent hanging in CI
#[cfg(feature = "pure-rust")]
mod device_tests {
    use keyrs_core::event::{EventLoop, EventLoopError};

    #[test]
    #[cfg(feature = "pure-rust")]
    #[ignore = "Requires actual keyboard devices"]
    fn test_event_loop_creation() {
        match EventLoop::new() {
            Ok(loop_) => {
                // Should find at least one keyboard
                assert!(loop_.device_count() > 0);
                println!("Found {} keyboard device(s)", loop_.device_count());
            }
            Err(EventLoopError::DeviceNotFound(_)) => {
                // No keyboard devices - skip test in CI environment
                println!("Skipping test: no keyboard devices found");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}

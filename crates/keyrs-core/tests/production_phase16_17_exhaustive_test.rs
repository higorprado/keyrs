#[cfg(feature = "pure-rust")]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use keyrs_core::config::Config;
    use keyrs_core::mapping::ActionStep;
    use keyrs_core::transform::engine::{TransformEngine, TransformResult};
    use keyrs_core::{Action, Key};

    fn load_engine() -> TransformEngine {
        let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/config-production-nonunicode.toml");
        let content = fs::read_to_string(config_path).expect("failed to read production config");
        let config = Config::from_toml(&content).expect("failed to parse production config");
        TransformEngine::new(config.to_transform_config())
    }

    fn press_combo(engine: &mut TransformEngine, mods: &[Key], key: Key) -> TransformResult {
        for m in mods {
            let _ = engine.process_event(*m, Action::Press);
        }
        let result = engine.process_event(key, Action::Press);
        for m in mods.iter().rev() {
            let _ = engine.process_event(*m, Action::Release);
        }
        let _ = engine.process_event(key, Action::Release);
        result
    }

    fn assert_combo_key(result: TransformResult, expected_key_code: u16) {
        match result {
            TransformResult::ComboKey(k) => assert_eq!(k.code(), expected_key_code),
            other => panic!("expected ComboKey({expected_key_code}), got {other:?}"),
        }
    }

    fn assert_combo_with_modifier(
        result: TransformResult,
        expected_key_code: u16,
        expected_modifier_key_code: u16,
    ) {
        let combo = match result {
            TransformResult::Combo(c) => c,
            other => panic!("expected Combo, got {other:?}"),
        };
        assert_eq!(combo.key().code(), expected_key_code);
        assert!(
            combo
                .modifiers()
                .iter()
                .any(|m| m.keys().iter().any(|k| k.code() == expected_modifier_key_code)),
            "expected modifier key code {} in combo {:?}",
            expected_modifier_key_code,
            combo
        );
    }

    fn assert_combo_with_modifiers(result: TransformResult, expected_key_code: u16, mods: &[u16]) {
        let combo = match result {
            TransformResult::Combo(c) => c,
            other => panic!("expected Combo, got {other:?}"),
        };
        assert_eq!(combo.key().code(), expected_key_code);
        for expected_mod in mods {
            assert!(
                combo
                    .modifiers()
                    .iter()
                    .any(|m| m.keys().iter().any(|k| k.code() == *expected_mod)),
                "expected modifier key code {} in combo {:?}",
                expected_mod,
                combo
            );
        }
    }

    fn assert_bound_combo(result: TransformResult, expected_key_code: u16, expected_mod: u16) {
        let steps = match result {
            TransformResult::Sequence(steps) => steps,
            other => panic!("expected Sequence for bind combo, got {other:?}"),
        };
        assert!(matches!(steps.first(), Some(ActionStep::Bind)));
        let combo = steps
            .iter()
            .find_map(|s| match s {
                ActionStep::Combo(c) => Some(c),
                _ => None,
            })
            .expect("missing combo step");
        assert_eq!(combo.key().code(), expected_key_code);
        assert!(
            combo
                .modifiers()
                .iter()
                .any(|m| m.keys().iter().any(|k| k.code() == expected_mod)),
            "expected modifier {} in {:?}",
            expected_mod,
            combo
        );
    }

    #[test]
    fn phase16_keymaps_are_present_in_config() {
        let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/config-production-nonunicode.toml");
        let content = fs::read_to_string(config_path).expect("failed to read production config");
        let config = Config::from_toml(&content).expect("failed to parse production config");

        let expected = [
            "prod_phase16_jdownloader",
            "prod_phase16_fileman_caja",
            "prod_phase16_fileman_cosmic_files",
            "prod_phase16_fileman_dde",
            "prod_phase16_fileman_dolphin_pre_kf6",
            "prod_phase16_fileman_dolphin_dialogs",
            "prod_phase16_fileman_dolphin",
            "prod_phase16_fileman_pantheon",
            "prod_phase16_fileman_krusader",
            "prod_phase16_fileman_nautilus_create_archive_dialog",
            "prod_phase16_fileman_nautilus",
            "prod_phase16_fileman_pcmanfmqt_desktop",
            "prod_phase16_fileman_pcmanfmqt",
            "prod_phase16_fileman_pcmanfm",
            "prod_phase16_fileman_peony_qt",
            "prod_phase16_fileman_spacefm_find_dialog",
            "prod_phase16_fileman_spacefm",
        ];
        for name in expected {
            assert!(
                config.keymaps.iter().any(|k| k.name == name),
                "missing keymap: {}",
                name
            );
        }
    }

    #[test]
    fn phase16_caja_ctrl_super_o_maps_to_shift_ctrl_enter() {
        let mut engine = load_engine();
        engine.update_window_context(Some("caja".to_string()), Some("Caja".to_string()));
        let result = press_combo(&mut engine, &[Key::from(29), Key::from(125)], Key::from(24)); // Ctrl+Super+O
        assert_combo_with_modifiers(result, 28, &[29, 42]); // Shift+Ctrl+Enter
    }

    #[test]
    fn phase16_cosmic_files_alt_enter_maps_to_space() {
        let mut engine = load_engine();
        engine.update_window_context(
            Some("com.system76.cosmicfiles".to_string()),
            Some("COSMIC Files".to_string()),
        );
        let result = press_combo(&mut engine, &[Key::from(56)], Key::from(28)); // Alt+Enter
        assert_combo_key(result, 57); // Space
    }

    #[test]
    fn phase16_dde_super_up_maps_to_ctrl_up() {
        let mut engine = load_engine();
        engine.update_window_context(Some("dde-file-manager".to_string()), Some("DDE".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(103)); // Super+Up
        assert_combo_with_modifier(result, 103, 29); // Ctrl+Up
    }

    #[test]
    fn phase16_pantheon_ctrl_super_o_maps_to_shift_enter() {
        let mut engine = load_engine();
        engine.update_window_context(Some("io.elementary.files".to_string()), Some("Files".to_string()));
        let result = press_combo(&mut engine, &[Key::from(29), Key::from(125)], Key::from(24)); // Ctrl+Super+O
        assert_combo_with_modifier(result, 28, 42); // Shift+Enter
    }

    #[test]
    fn phase16_krusader_super_2_maps_to_alt_shift_d() {
        let mut engine = load_engine();
        engine.update_window_context(Some("krusader".to_string()), Some("Krusader".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(3)); // Super+2
        assert_combo_with_modifiers(result, 32, &[56, 42]); // Alt+Shift+D
    }

    #[test]
    fn phase16_pcmanfmqt_super_1_sequence_shape() {
        let mut engine = load_engine();
        engine.update_window_context(Some("pcmanfm-qt".to_string()), Some("PCManFM-Qt".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(2)); // Super+1
        let steps = match result {
            TransformResult::Sequence(steps) => steps,
            other => panic!("expected Sequence, got {other:?}"),
        };
        assert_eq!(steps.len(), 5);
        assert!(matches!(steps[1], ActionStep::DelayMs(100)));
        assert!(matches!(steps[3], ActionStep::DelayMs(100)));
        assert!(matches!(steps[0], ActionStep::Combo(_)));
        assert!(matches!(steps[2], ActionStep::Combo(_)));
        assert!(matches!(steps[4], ActionStep::Combo(_)));
    }

    #[test]
    fn phase16_pcmanfm_super_2_maps_to_ctrl_4() {
        let mut engine = load_engine();
        engine.update_window_context(Some("pcmanfm".to_string()), Some("PCManFM".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(3)); // Super+2
        assert_combo_with_modifier(result, 5, 29); // Ctrl+4
    }

    #[test]
    fn phase16_peony_super_equal_maps_to_shift_ctrl_equal() {
        let mut engine = load_engine();
        engine.update_window_context(Some("peony-qt".to_string()), Some("Peony".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(13)); // Super+Equal
        assert_combo_with_modifiers(result, 13, &[29, 42]); // Shift+Ctrl+Equal
    }

    #[test]
    fn phase16_spacefm_super_page_down_binds_ctrl_tab() {
        let mut engine = load_engine();
        engine.update_window_context(Some("spacefm".to_string()), Some("SpaceFM".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(109)); // Super+Page_Down
        assert_bound_combo(result, 15, 29); // Ctrl+Tab
    }

    #[test]
    fn phase17_jetbrains_ctrl_w_maps_to_ctrl_f4() {
        let mut engine = load_engine();
        engine.update_window_context(
            Some("jetbrains-idea".to_string()),
            Some("IntelliJ IDEA".to_string()),
        );
        let result = press_combo(&mut engine, &[Key::from(29)], Key::from(17)); // Ctrl+W
        assert_combo_with_modifier(result, 62, 29); // Ctrl+F4
    }

    #[test]
    fn phase17_jetbrains_super_right_maps_to_alt_right() {
        let mut engine = load_engine();
        engine.update_window_context(
            Some("jetbrains-pycharm".to_string()),
            Some("PyCharm".to_string()),
        );
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(106)); // Super+Right
        assert_combo_with_modifier(result, 106, 56); // Alt+Right
    }

    #[test]
    fn phase17_jetbrains_super_v_maps_to_alt_grave() {
        let mut engine = load_engine();
        engine.update_window_context(
            Some("jetbrains-rustrover".to_string()),
            Some("RustRover".to_string()),
        );
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(47)); // Super+V
        assert_combo_with_modifier(result, 41, 56); // Alt+Grave
    }
}

#[cfg(feature = "pure-rust")]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use keyrs_core::config::Config;
    use keyrs_core::input::KeyboardType;
    use keyrs_core::mapping::ActionStep;
    use keyrs_core::settings::Settings;
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

    fn assert_bound_combo(
        result: TransformResult,
        expected_key_code: u16,
        expected_modifier_key_code: u16,
    ) {
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
                .any(|m| m.keys().iter().any(|k| k.code() == expected_modifier_key_code)),
            "expected modifier key code {} in combo {:?}",
            expected_modifier_key_code,
            combo
        );
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

    #[test]
    fn phase11_cmd_dot_not_terminals_active() {
        let mut engine = load_engine();
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(52)); // Super+Dot
        assert_combo_key(result, 1); // ESC
    }

    #[test]
    fn phase11_cmd_dot_not_terminals_inactive_in_terminal() {
        let mut engine = load_engine();
        engine.update_window_context(Some("kitty".to_string()), Some("terminal".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(52)); // Super+Dot
        assert!(
            !matches!(result, TransformResult::ComboKey(k) if k.code() == 1),
            "expected not to map to ESC in terminals"
        );
    }

    #[test]
    fn phase10b_genterms_ctrl_left_active_when_ubuntu_fedora_flag() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("DistroUbuntuOrFedoraGnome", true);
        engine.set_settings(settings);
        engine.update_window_context(Some("kitty".to_string()), Some("terminal".to_string()));
        let result = press_combo(&mut engine, &[Key::from(29)], Key::from(105)); // Ctrl+Left
        assert_bound_combo(result, 109, 125); // Super+Page_Down
    }

    #[test]
    fn phase10b_genterms_ctrl_left_inactive_without_flag() {
        let mut engine = load_engine();
        engine.update_window_context(Some("kitty".to_string()), Some("terminal".to_string()));
        let result = press_combo(&mut engine, &[Key::from(29)], Key::from(105)); // Ctrl+Left
        assert!(matches!(result, TransformResult::Passthrough(k) if k.code() == 105));
    }

    #[test]
    fn phase11_gengui_gnome_super_space_active_non_terminal() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("DesktopGnome", true);
        engine.set_settings(settings);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(57)); // Super+Space
        assert_combo_with_modifier(result, 57, 29); // Shift-Ctrl-Space
    }

    #[test]
    fn phase11_super_space_retriggers_while_super_is_held() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("DesktopBudgie", true);
        engine.set_settings(settings);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));

        let _super_press = engine.process_event(Key::from(125), Action::Press); // Super down
        let first = engine.process_event(Key::from(57), Action::Press); // Space press
        let _first_release = engine.process_event(Key::from(57), Action::Release); // Space release
        let second = engine.process_event(Key::from(57), Action::Press); // Space press again while Super held
        let _second_release = engine.process_event(Key::from(57), Action::Release);
        let _super_release = engine.process_event(Key::from(125), Action::Release);

        assert!(matches!(first, TransformResult::ComboKey(k) if k.code() == 125));
        assert!(matches!(second, TransformResult::ComboKey(k) if k.code() == 125));
    }

    #[test]
    fn phase11b_not_chromebook_super_tab_maps_to_ctrl_tab() {
        let mut engine = load_engine();
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(15)); // Super+Tab
        assert_bound_combo(result, 15, 29); // Ctrl+Tab
    }

    #[test]
    fn phase11b_chromebook_ibm_alt_tab_maps_to_ctrl_tab_on_ibm() {
        let mut engine = load_engine();
        engine.set_keyboard_type(KeyboardType::IBM);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(56)], Key::from(15)); // Alt+Tab
        assert_bound_combo(result, 15, 29); // Ctrl+Tab
    }

    #[test]
    fn phase12_general_gui_super_q_maps_to_alt_f4() {
        let mut engine = load_engine();
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(16)); // Super+Q
        assert_combo_with_modifier(result, 62, 56); // Alt+F4
    }

    #[test]
    fn phase12_general_gui_super_tab_respects_not_chromebook_override() {
        let mut engine = load_engine();
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(15)); // Super+Tab
        assert_bound_combo(result, 15, 29); // Ctrl+Tab (not-chromebook override)
    }

    #[test]
    fn phase12_general_gui_super_k_maps_to_shift_end_backspace_sequence() {
        let mut engine = load_engine();
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(37)); // Super+K
        let steps = match result {
            TransformResult::Sequence(steps) => steps,
            other => panic!("expected Sequence, got {other:?}"),
        };
        assert_eq!(steps.len(), 2);
        let first_combo = match &steps[0] {
            ActionStep::Combo(c) => c,
            other => panic!("expected first step Combo, got {other:?}"),
        };
        assert_eq!(first_combo.key().code(), 107); // End
        assert!(
            first_combo
                .modifiers()
                .iter()
                .any(|m| m.keys().iter().any(|k| k.code() == 42 || k.code() == 54)),
            "expected Shift modifier on first combo"
        );
        let second_combo = match &steps[1] {
            ActionStep::Combo(c) => c,
            other => panic!("expected second step Combo, got {other:?}"),
        };
        assert_eq!(second_combo.key().code(), 14); // Backspace
    }

    #[test]
    fn phase12_general_gui_ralt_enter_maps_to_insert() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("Enter2Ent_Cmd", true);
        engine.set_settings(settings);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(100)], Key::from(28)); // RAlt+Enter
        assert_combo_key(result, 110); // Insert
    }

    #[test]
    fn phase11_gengui_gnome_shift_super_3_maps_to_shift_print() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("DesktopGnome", true);
        engine.set_settings(settings);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125), Key::from(42)], Key::from(4)); // Shift+Super+3
        assert_combo_with_modifier(result, 99, 42); // Shift+Print(SysRq)
    }

    #[test]
    fn phase11b_gengui_zorin_xfce_super_space_maps_to_alt_pause() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("DistroZorinXfce", true);
        engine.set_settings(settings);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(57)); // Super+Space
        assert_combo_with_modifier(result, 119, 56); // Alt+Pause
    }

    #[test]
    fn phase11b_gengui_xfce_shift_super_4_maps_to_alt_print() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("DesktopXfce", true);
        engine.set_settings(settings);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125), Key::from(42)], Key::from(5)); // Shift+Super+4
        assert_combo_with_modifier(result, 99, 56); // Alt+Print(SysRq)
    }

    #[test]
    fn phase12_forced_numpad_alt_numlock_toggles_and_numlock_becomes_escape() {
        let mut engine = load_engine();
        engine.set_settings(Settings::new());
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));

        let toggle_on = press_combo(&mut engine, &[Key::from(56)], Key::from(69)); // Alt+NumLock
        assert!(matches!(toggle_on, TransformResult::Suppress));

        let numlock_as_clear = engine.process_event(Key::from(69), Action::Press); // NumLock
        assert_combo_key(numlock_as_clear, 1); // Escape
    }

    #[test]
    fn phase12_forced_numpad_toggle_off_restores_numlock_passthrough() {
        let mut engine = load_engine();
        engine.set_settings(Settings::new());
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));

        let _ = press_combo(&mut engine, &[Key::from(56)], Key::from(69)); // on
        let toggle_off = press_combo(&mut engine, &[Key::from(56)], Key::from(69)); // off
        assert!(matches!(toggle_off, TransformResult::Suppress));

        let numlock = engine.process_event(Key::from(69), Action::Press);
        assert!(matches!(numlock, TransformResult::Passthrough(k) if k.code() == 69));
    }

    #[test]
    fn phase13_media_arrows_fix_maps_playpause_to_page_up() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("media_arrows_fix", true);
        engine.set_settings(settings);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));

        let result = engine.process_event(Key::from(164), Action::Press); // PLAYPAUSE
        assert!(matches!(result, TransformResult::Remapped(k) if k.code() == 104)); // PAGE_UP
    }

    #[test]
    fn phase13_gtk3_numpad_nav_fix_maps_kp9_to_page_up_when_numlock_off() {
        let mut engine = load_engine();
        engine.set_settings(Settings::new());
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));

        let result = engine.process_event(Key::from(73), Action::Press); // KP9
        assert!(matches!(result, TransformResult::Remapped(k) if k.code() == 104)); // PAGE_UP
    }

    #[test]
    fn phase14_multipurpose_enter2cmd_tap_emits_enter() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("Enter2Ent_Cmd", true);
        engine.set_settings(settings);

        let press = engine.process_event(Key::from(28), Action::Press); // ENTER
        let release = engine.process_event(Key::from(28), Action::Release);
        assert!(matches!(press, TransformResult::Suppress));
        assert!(matches!(release, TransformResult::Remapped(k) if k.code() == 28));
    }

    #[test]
    fn phase14_multipurpose_caps2esc_not_chromebook_tap_emits_escape() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("Caps2Esc_Cmd", true);
        engine.set_settings(settings);
        engine.set_keyboard_type(KeyboardType::Mac);

        let press = engine.process_event(Key::from(58), Action::Press); // CAPSLOCK
        let release = engine.process_event(Key::from(58), Action::Release);
        assert!(matches!(press, TransformResult::Suppress));
        assert!(matches!(release, TransformResult::Remapped(k) if k.code() == 1)); // ESC
    }

    #[test]
    fn phase14_multipurpose_caps2esc_chromebook_uses_left_meta_trigger() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("Caps2Esc_Cmd", true);
        engine.set_settings(settings);
        engine.set_keyboard_type(KeyboardType::Chromebook);

        let press = engine.process_event(Key::from(125), Action::Press); // LEFT_META
        let release = engine.process_event(Key::from(125), Action::Release);
        assert!(matches!(press, TransformResult::Suppress));
        assert!(matches!(release, TransformResult::Remapped(k) if k.code() == 1)); // ESC
    }

    #[test]
    fn phase15_gui_windows_left_ctrl_maps_to_left_meta() {
        let mut engine = load_engine();
        engine.set_settings(Settings::new());
        engine.set_keyboard_type(KeyboardType::Windows);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));

        let result = engine.process_event(Key::from(29), Action::Press); // LEFT_CTRL
        assert!(matches!(result, TransformResult::Remapped(k) if k.code() == 125)); // LEFT_META
    }

    #[test]
    fn phase15_gui_windows_multi_lang_off_right_alt_maps_to_right_ctrl() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("multi_lang", false);
        engine.set_settings(settings);
        engine.set_keyboard_type(KeyboardType::Windows);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));

        let result = engine.process_event(Key::from(100), Action::Press); // RIGHT_ALT
        assert!(matches!(result, TransformResult::Remapped(k) if k.code() == 97)); // RIGHT_CTRL
    }

    #[test]
    fn phase15_terms_mac_left_meta_stays_left_meta() {
        let mut engine = load_engine();
        engine.set_settings(Settings::new());
        engine.set_keyboard_type(KeyboardType::Mac);
        engine.update_window_context(Some("kitty".to_string()), Some("terminal".to_string()));

        let result = engine.process_event(Key::from(125), Action::Press); // LEFT_META
        assert!(
            matches!(result, TransformResult::Remapped(k) if k.code() == 125)
                || matches!(result, TransformResult::Passthrough(k) if k.code() == 125)
        ); // LEFT_META
    }

    #[test]
    fn phase15_terms_mac_super_space_keeps_super_for_launcher() {
        let mut engine = load_engine();
        engine.set_settings(Settings::new());
        engine.set_keyboard_type(KeyboardType::Mac);
        engine.update_window_context(Some("kitty".to_string()), Some("terminal".to_string()));

        let super_press = engine.process_event(Key::from(125), Action::Press); // LEFT_META
        let space_press = engine.process_event(Key::from(57), Action::Press); // SPACE
        let space_release = engine.process_event(Key::from(57), Action::Release); // SPACE
        let super_release = engine.process_event(Key::from(125), Action::Release); // LEFT_META

        assert!(matches!(
            super_press,
            TransformResult::Remapped(k) if k.code() == 125
        ) || matches!(
            super_press,
            TransformResult::Passthrough(k) if k.code() == 125
        ));
        assert!(matches!(space_press, TransformResult::Passthrough(k) if k.code() == 57));
        assert!(matches!(space_release, TransformResult::Passthrough(k) if k.code() == 57));
        assert!(matches!(
            super_release,
            TransformResult::Remapped(k) if k.code() == 125
        ) || matches!(
            super_release,
            TransformResult::Passthrough(k) if k.code() == 125
        ));
    }

    #[test]
    fn phase15_caps2cmd_non_chromebook_maps_capslock_to_right_ctrl() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("Caps2Cmd", true);
        engine.set_settings(settings);
        engine.set_keyboard_type(KeyboardType::Windows);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));

        let result = engine.process_event(Key::from(58), Action::Press); // CAPSLOCK
        assert!(matches!(result, TransformResult::Remapped(k) if k.code() == 97)); // RIGHT_CTRL
    }

    #[test]
    fn phase15_caps2cmd_chromebook_maps_left_meta_to_right_ctrl() {
        let mut engine = load_engine();
        let mut settings = Settings::new();
        settings.set_bool("Caps2Cmd", true);
        engine.set_settings(settings);
        engine.set_keyboard_type(KeyboardType::Chromebook);
        engine.update_window_context(Some("firefox".to_string()), Some("Mozilla Firefox".to_string()));

        let result = engine.process_event(Key::from(125), Action::Press); // LEFT_META
        assert!(matches!(result, TransformResult::Remapped(k) if k.code() == 97)); // RIGHT_CTRL
    }

    #[test]
    fn phase16_jdownloader_super_comma_maps_to_ctrl_p() {
        let mut engine = load_engine();
        engine.update_window_context(Some("jdownloader".to_string()), Some("JDownloader".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(51)); // Super+Comma
        assert_combo_with_modifier(result, 25, 29); // Ctrl+P
    }

    #[test]
    fn phase16_nautilus_super_1_maps_to_ctrl_2() {
        let mut engine = load_engine();
        engine.update_window_context(Some("nautilus".to_string()), Some("Files".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(2)); // Super+1
        assert_combo_with_modifier(result, 3, 29); // Ctrl+2
    }

    #[test]
    fn phase16_dde_super_comma_is_ignored() {
        let mut engine = load_engine();
        engine.update_window_context(Some("dde-file-manager".to_string()), Some("DDE".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(51)); // Super+Comma
        assert!(matches!(result, TransformResult::Sequence(steps) if steps == vec![ActionStep::Ignore]));
    }

    #[test]
    fn phase16_spacefm_super_page_up_binds_ctrl_shift_tab() {
        let mut engine = load_engine();
        engine.update_window_context(Some("spacefm".to_string()), Some("SpaceFM".to_string()));
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(104)); // Super+Page_Up
        assert_bound_combo(result, 15, 29); // Ctrl+Tab family bind
    }

    #[test]
    fn phase17_jetbrains_ctrl_1_maps_to_alt_1() {
        let mut engine = load_engine();
        engine.update_window_context(
            Some("jetbrains-idea".to_string()),
            Some("IntelliJ IDEA".to_string()),
        );
        let result = press_combo(&mut engine, &[Key::from(29)], Key::from(2)); // Ctrl+1
        assert_combo_with_modifier(result, 2, 56); // Alt+1
    }

    #[test]
    fn phase17_jetbrains_ctrl_comma_maps_to_ctrl_alt_s() {
        let mut engine = load_engine();
        engine.update_window_context(
            Some("jetbrains-pycharm".to_string()),
            Some("PyCharm".to_string()),
        );
        let result = press_combo(&mut engine, &[Key::from(29)], Key::from(51)); // Ctrl+Comma
        let combo = match result {
            TransformResult::Combo(c) => c,
            other => panic!("expected Combo, got {other:?}"),
        };
        assert_eq!(combo.key().code(), 31); // S
        let has_ctrl = combo
            .modifiers()
            .iter()
            .any(|m| m.keys().iter().any(|k| k.code() == 29 || k.code() == 97));
        let has_alt = combo
            .modifiers()
            .iter()
            .any(|m| m.keys().iter().any(|k| k.code() == 56 || k.code() == 100));
        assert!(has_ctrl && has_alt, "expected Ctrl+Alt modifiers in {:?}", combo);
    }

    #[test]
    fn phase17_jetbrains_super_g_maps_to_alt_j() {
        let mut engine = load_engine();
        engine.update_window_context(
            Some("jetbrains-rustrover".to_string()),
            Some("RustRover".to_string()),
        );
        let result = press_combo(&mut engine, &[Key::from(125)], Key::from(34)); // Super+G
        assert_combo_with_modifier(result, 36, 56); // Alt+J
    }

    #[test]
    fn phase17_jetbrains_toolbox_is_excluded() {
        let mut engine = load_engine();
        engine.update_window_context(
            Some("jetbrains-toolbox".to_string()),
            Some("JetBrains Toolbox".to_string()),
        );
        let result = press_combo(&mut engine, &[Key::from(29)], Key::from(2)); // Ctrl+1
        assert!(matches!(result, TransformResult::Passthrough(k) if k.code() == 2));
    }
}

#[cfg(feature = "pure-rust")]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use keyrs_core::config::Config;

    #[test]
    fn user_overrides_template_parses_and_contains_expected_keymaps() {
        let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/config-user-overrides-template.toml");
        let content = fs::read_to_string(config_path).expect("failed to read template config");
        let config = Config::from_toml(&content).expect("failed to parse template config");

        assert_eq!(config.keymaps.len(), 3);
        assert!(config
            .keymaps
            .iter()
            .any(|k| k.name == "prod_user_hardware_keys"));
        assert!(config
            .keymaps
            .iter()
            .any(|k| k.name == "prod_user_overrides_terminals"));
        assert!(config
            .keymaps
            .iter()
            .any(|k| k.name == "prod_user_overrides_general"));

        for k in &config.keymaps {
            assert!(k.mappings.is_empty(), "template keymap should start empty: {}", k.name);
            assert!(k.condition.is_some(), "template keymap should be scoped: {}", k.name);
        }
    }
}

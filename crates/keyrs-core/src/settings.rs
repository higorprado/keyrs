// Keyrs Settings Module
// Handles user-configurable settings that can toggle features on/off

#![cfg(feature = "pure-rust")]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Settings for keyrs that control feature toggles
/// 
/// These settings are loaded from a TOML file (default: ~/.config/keyrs/settings.toml)
/// and can be used in conditions like:
///   condition = "settings.Enter2Ent_Cmd"
///   condition = "settings.Caps2Esc_Cmd and not settings.forced_numpad"
#[derive(Debug, Clone, Default)]
pub struct Settings {
    /// Feature toggles (e.g., Enter2Ent_Cmd, Caps2Esc_Cmd)
    features: HashMap<String, bool>,
    
    /// Layout setting (e.g., "ABC" or "US")
    optspec_layout: String,
    
    /// Keyboard type override (optional)
    keyboard_override: Option<String>,
    
    /// Path to the settings file (for reload)
    source_path: Option<PathBuf>,
}

/// Errors that can occur when loading settings
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parse error: {0}")]
    TomlParse(String),
    
    #[error("Invalid setting value: {0}")]
    InvalidValue(String),
}

/// TOML representation for deserializing settings
#[derive(Debug, Clone, serde::Deserialize, Default)]
struct SettingsToml {
    #[serde(default)]
    features: Option<HashMap<String, toml::Value>>,
    
    #[serde(default)]
    layout: Option<LayoutSettings>,
    
    #[serde(default)]
    keyboard: Option<KeyboardSettings>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct LayoutSettings {
    #[serde(default)]
    optspec_layout: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct KeyboardSettings {
    #[serde(default)]
    override_type: Option<String>,
}

impl Settings {
    /// Create a new empty settings object
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
            optspec_layout: "ABC".to_string(),
            keyboard_override: None,
            source_path: None,
        }
    }
    
    /// Load settings from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SettingsError> {
        let content = std::fs::read_to_string(&path)?;
        let mut settings = Self::from_toml(&content)?;
        settings.source_path = Some(path.as_ref().to_path_buf());
        Ok(settings)
    }
    
    /// Load settings from TOML string
    pub fn from_toml(content: &str) -> Result<Self, SettingsError> {
        let toml_settings: SettingsToml = toml::from_str(content)
            .map_err(|e| SettingsError::TomlParse(e.to_string()))?;
        
        let mut settings = Self::new();
        
        // Parse features section
        if let Some(features) = toml_settings.features {
            for (key, value) in features {
                let bool_value = parse_bool_value(&value)?;
                settings.features.insert(key, bool_value);
            }
        }
        
        // Parse layout section
        if let Some(layout) = toml_settings.layout {
            if let Some(optspec) = layout.optspec_layout {
                settings.optspec_layout = optspec;
            }
        }
        
        // Parse keyboard section
        if let Some(keyboard) = toml_settings.keyboard {
            settings.keyboard_override = keyboard.override_type;
        }
        
        Ok(settings)
    }
    
    /// Get the default settings path
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("keyrs").join("settings.toml"))
    }
    
    /// Load from default location (~/.config/keyrs/settings.toml)
    pub fn load_default() -> Result<Self, SettingsError> {
        if let Some(path) = Self::default_path() {
            if path.exists() {
                return Self::from_file(path);
            }
        }
        // Return default settings if file doesn't exist
        Ok(Self::new())
    }
    
    /// Get a boolean feature value
    pub fn get_bool(&self, name: &str) -> bool {
        self.features.get(name).copied().unwrap_or(false)
    }
    
    /// Set a boolean feature value
    pub fn set_bool(&mut self, name: &str, value: bool) {
        self.features.insert(name.to_string(), value);
    }
    
    /// Get the optspec layout
    pub fn optspec_layout(&self) -> &str {
        &self.optspec_layout
    }
    
    /// Get keyboard type override
    pub fn keyboard_override(&self) -> Option<&str> {
        self.keyboard_override.as_deref()
    }
    
    /// Check if a setting exists
    pub fn has_setting(&self, name: &str) -> bool {
        self.features.contains_key(name)
    }
    
    /// Get all features as a hashmap
    pub fn features(&self) -> &HashMap<String, bool> {
        &self.features
    }
    
    /// Reload settings from the original file
    pub fn reload(&mut self) -> Result<(), SettingsError> {
        if let Some(ref path) = self.source_path {
            let new_settings = Self::from_file(path)?;
            *self = new_settings;
            Ok(())
        } else {
            Err(SettingsError::InvalidValue("No source path set".to_string()))
        }
    }
    
    /// Evaluate a settings condition expression
    /// 
    /// Supports simple expressions like:
    ///   - "settings.Enter2Ent_Cmd" -> checks if that setting is true
    ///   - "settings.forced_numpad" -> checks if that setting is true
    /// 
    /// Note: This is a simplified evaluator. Complex expressions with
    /// "and", "or", "not" should be handled by the condition parser.
    pub fn evaluate_condition(&self, expr: &str) -> bool {
        // Handle simple "settings.X" pattern
        if let Some(setting_name) = expr.strip_prefix("settings.") {
            // Also handle negation like "not settings.X"
            let trimmed = setting_name.trim();
            return self.get_bool(trimmed);
        }
        
        // Handle "not settings.X" pattern
        if let Some(rest) = expr.strip_prefix("not settings.") {
            return !self.get_bool(rest.trim());
        }
        
        // Default: check if the setting exists and is true
        self.get_bool(expr)
    }
}

/// Parse a TOML value as a boolean
fn parse_bool_value(value: &toml::Value) -> Result<bool, SettingsError> {
    match value {
        toml::Value::Boolean(b) => Ok(*b),
        toml::Value::Integer(1) => Ok(true),
        toml::Value::Integer(0) => Ok(false),
        toml::Value::String(s) => {
            match s.to_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => Ok(true),
                "false" | "no" | "off" | "0" => Ok(false),
                _ => Err(SettingsError::InvalidValue(
                    format!("Cannot convert '{}' to boolean", s)
                )),
            }
        }
        _ => Err(SettingsError::InvalidValue(
            format!("Cannot convert {:?} to boolean", value)
        )),
    }
}

/// Create default settings content for a new installation
pub fn default_settings_content() -> &'static str {
    r#"# Keyrs Settings
# This file controls feature toggles for keyrs
# Place this file at: ~/.config/keyrs/settings.toml

[features]
# Enable/disable multipurpose modmaps
Enter2Ent_Cmd = false
Caps2Esc_Cmd = false
Caps2Cmd = false

# Enable/disable other features
media_arrows_fix = false
forced_numpad = false
multi_lang = false

[layout]
# Optional special character layout: "ABC" or "US"
optspec_layout = "ABC"

[keyboard]
# Optional keyboard type override (auto-detected if not set)
# Valid values: "IBM", "Chromebook", "Windows", "Apple"
# override_type = "Apple"
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::new();
        assert_eq!(settings.get_bool("Enter2Ent_Cmd"), false);
        assert_eq!(settings.optspec_layout(), "ABC");
    }

    #[test]
    fn test_settings_from_toml() {
        let toml = r#"
[features]
Enter2Ent_Cmd = true
Caps2Esc_Cmd = false
forced_numpad = true

[layout]
optspec_layout = "US"
"#;

        let settings = Settings::from_toml(toml).unwrap();
        assert_eq!(settings.get_bool("Enter2Ent_Cmd"), true);
        assert_eq!(settings.get_bool("Caps2Esc_Cmd"), false);
        assert_eq!(settings.get_bool("forced_numpad"), true);
        assert_eq!(settings.optspec_layout(), "US");
    }

    #[test]
    fn test_settings_evaluate_condition() {
        let mut settings = Settings::new();
        settings.set_bool("Enter2Ent_Cmd", true);
        settings.set_bool("Caps2Esc_Cmd", false);

        assert_eq!(settings.evaluate_condition("settings.Enter2Ent_Cmd"), true);
        assert_eq!(settings.evaluate_condition("settings.Caps2Esc_Cmd"), false);
        assert_eq!(settings.evaluate_condition("not settings.Caps2Esc_Cmd"), true);
    }

    #[test]
    fn test_settings_with_string_values() {
        let toml = r#"
[features]
Enter2Ent_Cmd = "true"
Caps2Esc_Cmd = "yes"
forced_numpad = "on"
"#;

        let settings = Settings::from_toml(toml).unwrap();
        assert_eq!(settings.get_bool("Enter2Ent_Cmd"), true);
        assert_eq!(settings.get_bool("Caps2Esc_Cmd"), true);
        assert_eq!(settings.get_bool("forced_numpad"), true);
    }

    #[test]
    fn test_keyboard_override() {
        let toml = r#"
[keyboard]
override_type = "Apple"
"#;

        let settings = Settings::from_toml(toml).unwrap();
        assert_eq!(settings.keyboard_override(), Some("Apple"));
    }
}

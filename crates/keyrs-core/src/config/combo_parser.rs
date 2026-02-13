// Keyrs Config API - Combo String Parser
// Parses combo strings like "Ctrl-Shift-A" into structured components

use crate::{Key, Modifier};
use std::collections::HashSet;

/// Result of parsing a combo string
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCombo {
    /// The modifiers parsed from the string (in order)
    pub modifiers: Vec<Modifier>,
    /// The key (the last component after hyphens)
    pub key: Key,
}

/// Errors that can occur during combo parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ComboParseError {
    /// Empty input string
    EmptyInput,
    /// Key name not recognized
    UnknownKey(String),
    /// Modifier alias not recognized
    UnknownModifier(String),
    /// Input ends with hyphen (e.g., "Ctrl-")
    TrailingHyphen,
}

impl std::fmt::Display for ComboParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComboParseError::EmptyInput => write!(f, "combo string cannot be empty"),
            ComboParseError::UnknownKey(name) => write!(f, "unknown key name: '{}'", name),
            ComboParseError::UnknownModifier(name) => write!(f, "unknown modifier: '{}'", name),
            ComboParseError::TrailingHyphen => write!(f, "combo string cannot end with hyphen"),
        }
    }
}

impl std::error::Error for ComboParseError {}

/// Parse a combo string like "Ctrl-Shift-A" into modifiers and key
///
/// # Arguments
/// * `exp` - The combo expression string to parse
///
/// # Returns
/// A `ParsedCombo` containing the modifiers and key
///
/// # Examples
/// ```
/// use keyrs_core::config::parse_combo_string;
/// use keyrs_core::Key;
/// let parsed = parse_combo_string("Ctrl-A").unwrap();
/// assert_eq!(parsed.modifiers.len(), 1);
/// assert_eq!(parsed.key, Key::from(30)); // Key::A
/// ```
pub fn parse_combo_string(exp: &str) -> Result<ParsedCombo, ComboParseError> {
    if exp.is_empty() {
        return Err(ComboParseError::EmptyInput);
    }

    let trimmed = exp.trim();
    if trimmed.is_empty() {
        return Err(ComboParseError::EmptyInput);
    }

    // Check for trailing hyphen
    if trimmed.ends_with('-') {
        return Err(ComboParseError::TrailingHyphen);
    }

    // Split by hyphens
    let parts: Vec<&str> = trimmed.split('-').collect();

    if parts.is_empty() {
        return Err(ComboParseError::EmptyInput);
    }

    // The last part is always the key
    let key_str = parts.last().unwrap();
    let key =
        key_from_name(key_str).ok_or_else(|| ComboParseError::UnknownKey(key_str.to_string()))?;

    // Everything before the last part are modifiers
    let mut modifiers = Vec::new();
    let mut seen_modifiers = HashSet::new();

    for (i, modifier_str) in parts.iter().enumerate() {
        // Skip the last part (it's the key)
        if i == parts.len() - 1 {
            break;
        }

        // Try to parse as a modifier alias
        let modifier = Modifier::from_alias(modifier_str)
            .ok_or_else(|| ComboParseError::UnknownModifier(modifier_str.to_string()))?;

        // Avoid duplicate modifiers
        if !seen_modifiers.contains(&modifier) {
            seen_modifiers.insert(modifier.clone());
            modifiers.push(modifier);
        }
    }

    Ok(ParsedCombo { modifiers, key })
}

/// Get a Key from its name
fn key_from_name(name: &str) -> Option<Key> {
    let upper = name.to_uppercase();
    // Try the key module's from_name function
    crate::key::key_from_name(&upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_key() {
        let parsed = parse_combo_string("a").unwrap();
        assert_eq!(parsed.modifiers.len(), 0);
        assert_eq!(parsed.key, Key::from(30)); // A key code
    }

    #[test]
    fn test_parse_single_modifier() {
        let parsed = parse_combo_string("Ctrl-a").unwrap();
        assert_eq!(parsed.modifiers.len(), 1);
        assert_eq!(parsed.modifiers[0].primary_alias(), "Ctrl");
        assert_eq!(parsed.key, Key::from(30)); // A key code
    }

    #[test]
    fn test_parse_multiple_modifiers() {
        let parsed = parse_combo_string("Ctrl-Shift-A").unwrap();
        assert_eq!(parsed.modifiers.len(), 2);
        assert_eq!(parsed.key, Key::from(30)); // A key code
    }

    #[test]
    fn test_parse_all_modifiers() {
        let parsed = parse_combo_string("Ctrl-Shift-Alt-Meta-A").unwrap();
        assert_eq!(parsed.modifiers.len(), 4);
    }

    #[test]
    fn test_parse_with_alias_variants() {
        // Test various modifier aliases
        let parsed1 = parse_combo_string("C-a").unwrap();
        assert_eq!(parsed1.modifiers.len(), 1);

        let parsed2 = parse_combo_string("LC-a").unwrap();
        assert_eq!(parsed2.modifiers.len(), 1);
        assert_eq!(parsed2.modifiers[0].primary_alias(), "LCtrl");

        let parsed3 = parse_combo_string("RC-a").unwrap();
        assert_eq!(parsed3.modifiers.len(), 1);
        assert_eq!(parsed3.modifiers[0].primary_alias(), "RCtrl");
    }

    #[test]
    fn test_parse_case_variants() {
        // Test various valid case formats for modifiers
        // Note: Modifier aliases have specific casing in the registry
        let parsed1 = parse_combo_string("Ctrl-A").unwrap();
        let parsed2 = parse_combo_string("C-A").unwrap(); // Short alias

        // Both should parse successfully with the same number of modifiers
        assert_eq!(parsed1.modifiers.len(), 1);
        assert_eq!(parsed2.modifiers.len(), 1);
        assert_eq!(parsed1.key, parsed2.key);
    }

    #[test]
    fn test_parse_empty_string() {
        let result = parse_combo_string("");
        assert_eq!(result, Err(ComboParseError::EmptyInput));
    }

    #[test]
    fn test_parse_whitespace_only() {
        let result = parse_combo_string("   ");
        assert_eq!(result, Err(ComboParseError::EmptyInput));
    }

    #[test]
    fn test_parse_trailing_hyphen() {
        let result = parse_combo_string("Ctrl-");
        assert_eq!(result, Err(ComboParseError::TrailingHyphen));
    }

    #[test]
    fn test_parse_unknown_key() {
        let result = parse_combo_string("Ctrl-NotARealKey");
        assert!(matches!(result, Err(ComboParseError::UnknownKey(_))));
    }

    #[test]
    fn test_parse_unknown_modifier() {
        let result = parse_combo_string("NotAModifier-A");
        assert!(matches!(result, Err(ComboParseError::UnknownModifier(_))));
    }

    #[test]
    fn test_parse_duplicate_modifiers() {
        // Duplicate modifiers should be deduplicated
        let parsed = parse_combo_string("Ctrl-Ctrl-A").unwrap();
        assert_eq!(parsed.modifiers.len(), 1);
    }

    #[test]
    fn test_parse_special_keys() {
        let parsed = parse_combo_string("Ctrl-ENTER").unwrap();
        assert_eq!(parsed.modifiers.len(), 1);
        assert_eq!(parsed.key, Key::from(28)); // ENTER key code

        let parsed2 = parse_combo_string("Shift-F1").unwrap();
        assert_eq!(parsed2.modifiers.len(), 1);
        assert_eq!(parsed2.key, Key::from(59)); // F1 key code
    }

    #[test]
    fn test_parse_with_whitespace() {
        let parsed = parse_combo_string("  Ctrl-A  ").unwrap();
        assert_eq!(parsed.modifiers.len(), 1);
        assert_eq!(parsed.key, Key::from(30));
    }

    #[test]
    fn test_cmd_alias_parsing() {
        // Test that "Cmd" modifier alias works
        let result = parse_combo_string("Cmd-c");
        assert!(result.is_ok(), "Cmd-c should parse: {:?}", result.err());

        let parsed = result.unwrap();
        assert_eq!(parsed.modifiers.len(), 1, "Should have 1 modifier");
        // The Cmd alias should resolve to a valid modifier (META/Super)
        let primary = parsed.modifiers[0].primary_alias();
        assert!(
            !primary.is_empty(),
            "Modifier should have a valid primary alias"
        );
    }

    #[test]
    fn test_cmd_case_sensitivity() {
        // Modifiers are case-sensitive: "Cmd" works but "cmd" doesn't
        let result_correct = parse_combo_string("Cmd-c");
        assert!(result_correct.is_ok(), "Cmd-c (correct case) should parse");

        let result_lowercase = parse_combo_string("cmd-c");
        assert!(
            result_lowercase.is_err(),
            "cmd-c (lowercase) should NOT parse - modifiers are case-sensitive"
        );
    }

    #[test]
    fn test_all_common_modifier_aliases() {
        // Test all common modifier aliases used in configs
        let test_cases = vec![
            ("Ctrl-c", true),
            ("Cmd-c", true),
            ("Super-c", true),
            ("Meta-c", true),
            ("Alt-c", true),
            ("Shift-c", true),
            ("Ctrl-Shift-c", true),
            ("Cmd-Shift-c", true),
            ("NotAMod-c", false), // Should fail
        ];

        for (combo_str, should_pass) in test_cases {
            let result = parse_combo_string(combo_str);
            if should_pass {
                assert!(
                    result.is_ok(),
                    "'{}' should parse successfully, got: {:?}",
                    combo_str,
                    result.err()
                );
            } else {
                assert!(
                    result.is_err(),
                    "'{}' should fail to parse, but parsed successfully",
                    combo_str
                );
            }
        }
    }

    #[test]
    fn test_various_key_names_with_cmd() {
        // Test various key names with Cmd modifier
        let test_cases = vec![
            "Cmd-a", "Cmd-c", "Cmd-v", "Cmd-x", "Cmd-z", "Cmd-ENTER", "Cmd-ESC", "Cmd-TAB",
            "Cmd-SPACE",
        ];

        for combo_str in test_cases {
            let result = parse_combo_string(combo_str);
            assert!(
                result.is_ok(),
                "'{}' should parse successfully, got: {:?}",
                combo_str,
                result.err()
            );
        }
    }
}

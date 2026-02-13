// Keyrs Config API - Keymap Modifier Expander
// Expands non-specific modifiers (Ctrl -> Left_Ctrl + Right_Ctrl)

use crate::{Combo, Modifier};

/// Expand keymap mappings by converting non-specific modifiers to specific ones
///
/// This function takes a list of (combo_string, key_code) tuples and expands
/// any combos that contain non-specific modifiers.
///
/// For example:
/// - "Ctrl-A" -> [("Left_Ctrl-A", key_code), ("Right_Ctrl-A", key_code)]
/// - "Ctrl-Shift-B" -> 4 variants with all L/R combinations
///
/// # Arguments
/// * `entries` - List of (combo_string, key_code) tuples
///
/// # Returns
/// A list of (combo_string, key_code) tuples with all specific modifiers
pub fn expand_keymap_entries(entries: &[(String, u16)]) -> Vec<(String, u16)> {
    let mut result = Vec::new();

    for (combo_str, key_code) in entries {
        // Parse the combo string
        if let Ok(parsed) = super::combo_parser::parse_combo_string(combo_str) {
            // Create a combo from the parsed result
            let combo = Combo::new(parsed.modifiers, parsed.key);

            // Expand the combo
            let expanded_combos = expand_combo(&combo);

            for expanded_combo in expanded_combos {
                // Convert back to string representation
                let combo_str = combo_to_string(&expanded_combo);
                result.push((combo_str, *key_code));
            }
        } else {
            // If parsing fails, keep the original
            result.push((combo_str.clone(), *key_code));
        }
    }

    result
}

/// Expand a single combo into all its specific modifier variants
///
/// For example, "Ctrl-Shift-A" becomes:
/// - Left_Ctrl-Left_Shift-A
/// - Left_Ctrl-Right_Shift-A
/// - Right_Ctrl-Left_Shift-A
/// - Right_Ctrl-Right_Shift-A
pub fn expand_combo(combo: &Combo) -> Vec<Combo> {
    let mut modifier_variants: Vec<Vec<Modifier>> = vec![Vec::new()];

    for modifier in combo.modifiers() {
        if modifier.is_specific() {
            // Specific modifier - add to all existing variants
            for variant in &mut modifier_variants {
                variant.push(modifier.clone());
            }
        } else {
            // Non-specific modifier - create left/right variants
            let left = modifier.to_left();
            let right = modifier.to_right();

            match (left, right) {
                (Some(l), Some(r)) => {
                    // Both variants exist - expand all current variants
                    let mut new_variants = Vec::new();
                    for variant in &modifier_variants {
                        let mut with_left: Vec<Modifier> = variant.clone();
                        with_left.push(l.clone());
                        new_variants.push(with_left);

                        let mut with_right: Vec<Modifier> = variant.clone();
                        with_right.push(r.clone());
                        new_variants.push(with_right);
                    }
                    modifier_variants = new_variants;
                }
                (Some(l), None) => {
                    // Only left variant exists
                    for variant in &mut modifier_variants {
                        variant.push(l.clone());
                    }
                }
                (None, Some(r)) => {
                    // Only right variant exists
                    for variant in &mut modifier_variants {
                        variant.push(r.clone());
                    }
                }
                (None, None) => {
                    // No variants - skip this modifier
                }
            }
        }
    }

    // Create combo for each modifier variant
    modifier_variants
        .into_iter()
        .map(|modifiers| Combo::new(modifiers, combo.key()))
        .collect()
}

/// Convert a combo back to string representation
fn combo_to_string(combo: &Combo) -> String {
    let mut parts = Vec::new();

    for modifier in combo.modifiers() {
        parts.push(modifier.primary_alias().to_string());
    }

    parts.push(combo.key().name().to_string());

    parts.join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Key;

    #[test]
    fn test_expand_specific_modifiers_only() {
        // No expansion needed
        let lctrl = Modifier::from_alias("LCtrl").unwrap();
        let lshift = Modifier::from_alias("LShift").unwrap();
        let combo = Combo::new(vec![lctrl, lshift], Key::from(30)); // A

        let expanded = expand_combo(&combo);
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0].modifiers().len(), 2);
    }

    #[test]
    fn test_expand_single_non_specific_modifier() {
        // Ctrl-A -> Left_Ctrl-A, Right_Ctrl-A
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::new(vec![ctrl], Key::from(30)); // A

        let expanded = expand_combo(&combo);
        assert_eq!(expanded.len(), 2);

        // Check that we have left and right variants
        let names: Vec<&str> = expanded
            .iter()
            .flat_map(|c| c.modifiers().iter().map(|m| m.primary_alias()))
            .collect();
        assert!(names.contains(&"LCtrl"));
        assert!(names.contains(&"RCtrl"));
    }

    #[test]
    fn test_expand_two_non_specific_modifiers() {
        // Ctrl-Shift-A -> 4 variants
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let shift = Modifier::from_alias("Shift").unwrap();
        let combo = Combo::new(vec![ctrl, shift], Key::from(30)); // A

        let expanded = expand_combo(&combo);
        assert_eq!(expanded.len(), 4); // 2 x 2 = 4
    }

    #[test]
    fn test_expand_mixed_modifiers() {
        // Ctrl-Left_Shift-A -> 2 variants (Ctrl expands, LShift doesn't)
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let lshift = Modifier::from_alias("LShift").unwrap();
        let combo = Combo::new(vec![ctrl, lshift], Key::from(30)); // A

        let expanded = expand_combo(&combo);
        assert_eq!(expanded.len(), 2);

        // All should have LShift
        for exp in &expanded {
            let has_lshift = exp
                .modifiers()
                .iter()
                .any(|m| m.primary_alias() == "LShift");
            assert!(has_lshift);
        }
    }

    #[test]
    fn test_expand_three_non_specific_modifiers() {
        // Ctrl-Shift-Alt-A -> 8 variants
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let shift = Modifier::from_alias("Shift").unwrap();
        let alt = Modifier::from_alias("Alt").unwrap();
        let combo = Combo::new(vec![ctrl, shift, alt], Key::from(30)); // A

        let expanded = expand_combo(&combo);
        assert_eq!(expanded.len(), 8); // 2 x 2 x 2 = 8
    }

    #[test]
    fn test_expand_no_modifiers() {
        // Just a key, no modifiers
        let combo = Combo::new(vec![], Key::from(30)); // A

        let expanded = expand_combo(&combo);
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0].modifiers().len(), 0);
    }

    #[test]
    fn test_expand_keymap_entries_simple() {
        // Test expanding a list of simple keymap entries
        let entries = vec![
            ("Ctrl-A".to_string(), 32), // Maps to C
            ("Ctrl-B".to_string(), 33), // Maps to D
        ];

        let expanded = expand_keymap_entries(&entries);

        // Each combo with "Ctrl" should expand to 2 variants
        assert_eq!(expanded.len(), 4);
    }

    #[test]
    fn test_combo_to_string() {
        let lctrl = Modifier::from_alias("LCtrl").unwrap();
        let combo = Combo::new(vec![lctrl], Key::from(30)); // A

        let s = combo_to_string(&combo);
        assert_eq!(s, "LCtrl-A");
    }

    #[test]
    fn test_combo_to_string_multiple_modifiers() {
        let lctrl = Modifier::from_alias("LCtrl").unwrap();
        let lshift = Modifier::from_alias("LShift").unwrap();
        let combo = Combo::new(vec![lctrl, lshift], Key::from(30)); // A

        let s = combo_to_string(&combo);
        assert_eq!(s, "LCtrl-LShift-A");
    }
}

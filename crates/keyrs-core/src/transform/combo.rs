// Xwaykeyz Transform Combo Matching
// Core combo matching logic

use crate::{Combo, ComboHint, Key, Keymap, KeymapValue, Modifier};

/// Result of a combo match operation
#[derive(Debug, Clone, PartialEq)]
pub enum ComboMatchResult {
    /// No combo found
    NotFound,
    /// Found a combo with a specific key output
    FoundKey(Key),
    /// Found a combo with a combo output
    FoundCombo(Combo),
    /// Found a combo with a multi-step sequence output
    FoundSequence(Vec<crate::mapping::ActionStep>),
    /// Found a combo with a hint output
    FoundHint(ComboHint),
    /// Found a combo with Unicode output
    FoundUnicode(u32),
    /// Found a combo with text output
    FoundText(String),
}

/// Try to find a matching combo in the keymaps
///
/// # Arguments
/// * `modifiers` - Slice of currently pressed modifiers
/// * `key` - The key being pressed
/// * `keymaps` - Slice of keymaps to search
///
/// # Returns
/// A `ComboMatchResult` indicating what was found
pub fn find_combo_match(modifiers: &[Modifier], key: Key, keymaps: &[Keymap]) -> ComboMatchResult {
    let combo = Combo::new(modifiers.to_vec(), key);

    for keymap in keymaps {
        if let Some(value) = keymap.get(&combo) {
            return match value {
                KeymapValue::Key(k) => ComboMatchResult::FoundKey(*k),
                KeymapValue::Combo(c) => ComboMatchResult::FoundCombo(c.clone()),
                KeymapValue::Sequence(steps) => ComboMatchResult::FoundSequence(steps.clone()),
                KeymapValue::ComboHint(h) => ComboMatchResult::FoundHint(*h),
                KeymapValue::Unicode(codepoint) => ComboMatchResult::FoundUnicode(*codepoint),
                KeymapValue::Text(text) => ComboMatchResult::FoundText(text.clone()),
            };
        }
    }

    ComboMatchResult::NotFound
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_combo_match_not_found() {
        let modifiers = vec![];
        let key = Key::from(30); // A
        let keymaps = vec![Keymap::new("test")];
        let result = find_combo_match(&modifiers, key, &keymaps);
        assert_eq!(result, ComboMatchResult::NotFound);
    }

    #[test]
    fn test_find_combo_match_found_key() {
        let mut keymap = Keymap::new("test");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::from_single(ctrl.clone(), Key::from(30)); // Ctrl-A
        keymap.insert(combo, Key::from(31).into()); // -> S

        let modifiers = vec![ctrl];
        let key = Key::from(30);
        let result = find_combo_match(&modifiers, key, &[keymap]);

        assert_eq!(result, ComboMatchResult::FoundKey(Key::from(31)));
    }

    #[test]
    fn test_find_combo_match_found_unicode() {
        let mut keymap = Keymap::new("test");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::from_single(ctrl.clone(), Key::from(18)); // Ctrl-E
        keymap.insert(combo, KeymapValue::Unicode(0x00E9)); // é

        let modifiers = vec![ctrl];
        let key = Key::from(18);
        let result = find_combo_match(&modifiers, key, &[keymap]);

        assert_eq!(result, ComboMatchResult::FoundUnicode(0x00E9));
    }

    #[test]
    fn test_find_combo_match_found_text() {
        let mut keymap = Keymap::new("test");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::from_single(ctrl.clone(), Key::from(20)); // Ctrl-T
        keymap.insert(combo, KeymapValue::Text("hello".to_string()));

        let modifiers = vec![ctrl];
        let key = Key::from(20);
        let result = find_combo_match(&modifiers, key, &[keymap]);

        assert_eq!(result, ComboMatchResult::FoundText("hello".to_string()));
    }

    #[test]
    fn test_find_combo_match_found_sequence() {
        let mut keymap = Keymap::new("test");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::from_single(ctrl.clone(), Key::from(20)); // Ctrl-T
        keymap.insert(
            combo,
            KeymapValue::Sequence(vec![
                crate::mapping::ActionStep::DelayMs(10),
                crate::mapping::ActionStep::Text("x".to_string()),
            ]),
        );

        let modifiers = vec![ctrl];
        let key = Key::from(20);
        let result = find_combo_match(&modifiers, key, &[keymap]);

        assert_eq!(
            result,
            ComboMatchResult::FoundSequence(vec![
                crate::mapping::ActionStep::DelayMs(10),
                crate::mapping::ActionStep::Text("x".to_string())
            ])
        );
    }
}

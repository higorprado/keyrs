// Keyrs Combo Sending Calculation
// Modifier arithmetic logic for determining which keys to lift/press

use crate::{Key, Modifier};

/// Sequence of actions to send for a combo
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComboActionSequence {
    /// Modifier keys to release (in reverse order - rightmost first)
    pub modifiers_to_release: Vec<Key>,
    /// Modifier keys to press
    pub modifiers_to_press: Vec<Key>,
    /// The main key of the combo
    pub main_key: Key,
    /// Modifier keys that were released and should be re-pressed after
    pub modifiers_to_restore: Vec<Key>,
}

impl ComboActionSequence {
    /// Create a new empty action sequence
    pub fn new() -> Self {
        Self {
            modifiers_to_release: Vec::new(),
            modifiers_to_press: Vec::new(),
            main_key: Key::from(0),
            modifiers_to_restore: Vec::new(),
        }
    }

    /// Create a new action sequence with all fields
    pub fn with_fields(release: Vec<Key>, press: Vec<Key>, main: Key, restore: Vec<Key>) -> Self {
        Self {
            modifiers_to_release: release,
            modifiers_to_press: press,
            main_key: main,
            modifiers_to_restore: restore,
        }
    }

    /// Check if this sequence requires any modifier changes
    pub fn needs_modifier_changes(&self) -> bool {
        !self.modifiers_to_release.is_empty()
            || !self.modifiers_to_press.is_empty()
            || !self.modifiers_to_restore.is_empty()
    }

    /// Get the total number of actions in this sequence
    pub fn total_actions(&self) -> usize {
        self.modifiers_to_release.len()
            + self.modifiers_to_press.len()
            + 2 // press and release main key
            + self.modifiers_to_restore.len()
    }
}

impl Default for ComboActionSequence {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate the sequence of actions needed to send a combo
///
/// This implements the modifier arithmetic logic from output.py lines 204-241.
/// It determines which modifier keys need to be lifted, pressed, and restored
/// based on the current pressed state and the desired combo.
///
/// # Algorithm
/// 1. Start with all currently pressed modifier keys in "need to lift" set
/// 2. For each pressed modifier key, check if it satisfies any combo modifier
/// 3. If it does, remove from "need to lift" and remove that combo modifier from "need to press"
/// 4. The result is: lift unneeded modifiers, press needed modifiers, send key, restore
///
/// # Arguments
/// * `combo_modifiers` - Modifiers required by the combo
/// * `combo_key` - The main key of the combo
/// * `pressed_modifier_keys` - Currently pressed modifier keys
///
/// # Returns
/// A `ComboActionSequence` containing the keys to release, press, and restore
pub fn calculate_combo_actions(
    combo_modifiers: &[Modifier],
    combo_key: Key,
    pressed_modifier_keys: &[Key],
) -> ComboActionSequence {
    // Start with all pressed modifiers in the "need to lift" set
    let mut mod_keys_to_lift: Vec<Key> = pressed_modifier_keys.to_vec();
    let mut mods_to_press: Vec<Modifier> = combo_modifiers.to_vec();

    // Check if any pressed modifier satisfies a combo modifier
    for pressed_key in pressed_modifier_keys {
        for modifier in combo_modifiers {
            if modifier.keys().contains(pressed_key) {
                // This modifier is already held, don't need to lift or press
                mod_keys_to_lift.retain(|k| k != pressed_key);

                // Remove from mods_to_press if present
                // Handle case where same mod appears twice (e.g., both left/right in input)
                if let Some(pos) = mods_to_press.iter().position(|m| m == modifier) {
                    mods_to_press.remove(pos);
                }
            }
        }
    }

    // Build the action sequence
    let modifiers_to_release: Vec<Key> = mod_keys_to_lift.into_iter().rev().collect();
    let modifiers_to_press = mods_to_press.iter().map(|m| m.key()).collect();
    // Lifted modifiers must be restored after combo emission to preserve
    // physical hold semantics (e.g. holding Super while tapping Space repeatedly).
    let modifiers_to_restore = modifiers_to_release.iter().rev().copied().collect();

    ComboActionSequence::with_fields(
        modifiers_to_release,
        modifiers_to_press,
        combo_key,
        modifiers_to_restore,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combo_no_modifiers() {
        let combo_modifiers = vec![];
        let combo_key = Key::from(30); // A
        let pressed_mods = vec![];

        let result = calculate_combo_actions(&combo_modifiers, combo_key, &pressed_mods);

        assert!(result.modifiers_to_release.is_empty());
        assert!(result.modifiers_to_press.is_empty());
        assert_eq!(result.main_key, combo_key);
    }

    #[test]
    fn test_combo_all_modifiers_pressed() {
        // Ctrl-A combo when Ctrl is already pressed
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let combo_modifiers = vec![ctrl.clone()];
        let combo_key = Key::from(30); // A
        let left_ctrl = Key::from(29); // LEFT_CTRL
        let pressed_mods = vec![left_ctrl];

        let result = calculate_combo_actions(&combo_modifiers, combo_key, &pressed_mods);

        // No need to lift or press - Ctrl is already held
        assert!(result.modifiers_to_release.is_empty());
        assert!(result.modifiers_to_press.is_empty());
        assert_eq!(result.main_key, combo_key);
    }

    #[test]
    fn test_combo_lift_some_press_some() {
        // Ctrl-A combo when Shift is pressed
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let combo_modifiers = vec![ctrl.clone()];
        let combo_key = Key::from(30); // A
        let left_shift = Key::from(42); // LEFT_SHIFT
        let pressed_mods = vec![left_shift];

        let result = calculate_combo_actions(&combo_modifiers, combo_key, &pressed_mods);

        // Need to lift Shift, press Ctrl
        assert_eq!(result.modifiers_to_release, vec![left_shift]);
        assert_eq!(result.modifiers_to_press, vec![ctrl.key()]);
        assert_eq!(result.main_key, combo_key);
    }

    #[test]
    fn test_combo_no_modifiers_pressed() {
        // Ctrl-A combo when no modifiers are pressed
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let combo_modifiers = vec![ctrl.clone()];
        let combo_key = Key::from(30); // A
        let pressed_mods = vec![];

        let result = calculate_combo_actions(&combo_modifiers, combo_key, &pressed_mods);

        // Just need to press Ctrl
        assert!(result.modifiers_to_release.is_empty());
        assert_eq!(result.modifiers_to_press, vec![ctrl.key()]);
        assert_eq!(result.main_key, combo_key);
    }

    #[test]
    fn test_combo_left_vs_right_modifiers() {
        // Test generic Ctrl with left Ctrl pressed
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let combo_modifiers = vec![ctrl.clone()];
        let combo_key = Key::from(30); // A
        let left_ctrl = Key::from(29); // LEFT_CTRL
        let pressed_mods = vec![left_ctrl];

        let result = calculate_combo_actions(&combo_modifiers, combo_key, &pressed_mods);

        // Left Ctrl satisfies the generic Ctrl requirement
        assert!(result.modifiers_to_release.is_empty());
        assert!(result.modifiers_to_press.is_empty());
    }

    #[test]
    fn test_combo_multiple_modifiers_partial() {
        // Ctrl-Shift-A when only Shift is pressed
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let shift = Modifier::from_name("SHIFT").unwrap();
        let combo_modifiers = vec![ctrl.clone(), shift.clone()];
        let combo_key = Key::from(30); // A
        let left_shift = Key::from(42); // LEFT_SHIFT
        let pressed_mods = vec![left_shift];

        let result = calculate_combo_actions(&combo_modifiers, combo_key, &pressed_mods);

        // Need to press Ctrl, Shift is already pressed
        assert!(result.modifiers_to_release.is_empty());
        assert_eq!(result.modifiers_to_press, vec![ctrl.key()]);
    }

    #[test]
    fn test_combo_multiple_modifiers_all_pressed() {
        // Ctrl-Shift-A when both are already pressed
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let shift = Modifier::from_name("SHIFT").unwrap();
        let combo_modifiers = vec![ctrl.clone(), shift.clone()];
        let combo_key = Key::from(30); // A
        let left_ctrl = Key::from(29);
        let left_shift = Key::from(42);
        let pressed_mods = vec![left_ctrl, left_shift];

        let result = calculate_combo_actions(&combo_modifiers, combo_key, &pressed_mods);

        // All modifiers already pressed
        assert!(result.modifiers_to_release.is_empty());
        assert!(result.modifiers_to_press.is_empty());
    }

    #[test]
    fn test_combo_conflicting_modifiers() {
        // Ctrl-A when Alt is pressed (need to lift Alt, press Ctrl)
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let combo_modifiers = vec![ctrl.clone()];
        let combo_key = Key::from(30); // A
        let left_alt = Key::from(56); // LEFT_ALT
        let pressed_mods = vec![left_alt];

        let result = calculate_combo_actions(&combo_modifiers, combo_key, &pressed_mods);

        assert_eq!(result.modifiers_to_release, vec![left_alt]);
        assert_eq!(result.modifiers_to_press, vec![ctrl.key()]);
        assert_eq!(result.modifiers_to_restore, vec![left_alt]);
    }

    #[test]
    fn test_combo_restores_lifted_modifiers_in_original_order() {
        // Ctrl-A when Ctrl+Shift are both held and only Ctrl is needed.
        // Shift should be lifted then restored.
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let combo_modifiers = vec![ctrl.clone()];
        let combo_key = Key::from(30); // A
        let left_ctrl = Key::from(29);
        let left_shift = Key::from(42);
        let pressed_mods = vec![left_ctrl, left_shift];

        let result = calculate_combo_actions(&combo_modifiers, combo_key, &pressed_mods);
        assert_eq!(result.modifiers_to_release, vec![left_shift]);
        assert_eq!(result.modifiers_to_restore, vec![left_shift]);
    }

    #[test]
    fn test_combo_action_sequence_defaults() {
        let seq = ComboActionSequence::new();
        assert!(seq.modifiers_to_release.is_empty());
        assert!(seq.modifiers_to_press.is_empty());
        assert!(seq.modifiers_to_restore.is_empty());
        assert!(!seq.needs_modifier_changes());

        let seq_default = ComboActionSequence::default();
        assert!(seq_default.modifiers_to_release.is_empty());
    }

    #[test]
    fn test_combo_action_sequence_needs_changes() {
        let key = Key::from(30);
        let seq = ComboActionSequence::with_fields(vec![key], vec![], Key::from(31), vec![]);
        assert!(seq.needs_modifier_changes());
        assert_eq!(seq.total_actions(), 3); // release + press + release main
    }

    #[test]
    fn test_combo_restore_field() {
        let key = Key::from(30);
        let seq = ComboActionSequence::with_fields(
            vec![key],
            vec![],
            Key::from(31),
            vec![key], // Will restore
        );
        assert_eq!(seq.modifiers_to_restore, vec![key]);
    }
}

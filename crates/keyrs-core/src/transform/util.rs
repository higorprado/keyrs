// Keyrs Transform Utility Functions
// Pure functions for querying key state

use crate::Key;
use crate::Keystate;
use crate::Modifier;
use smallvec::SmallVec;

/// Get current modifier state as a sorted vector of Key codes.
///
/// This returns a hashable representation for cache comparison.
/// Returns pressed modifier Key codes, sorted by value for consistent comparison.
/// Uses SmallVec to avoid heap allocation for the common case of 0-4 modifiers.
///
/// # Arguments
/// * `key_states` - Slice of all current keystates
///
/// # Returns
/// A vector of Key codes (u16) for pressed modifiers, sorted by code value
pub fn get_modifier_snapshot(key_states: &[Keystate]) -> SmallVec<[u16; 4]> {
    let mut mod_keys: SmallVec<[u16; 4]> = key_states
        .iter()
        .filter(|ks| ks.key_is_pressed())
        .map(|ks| ks.key.unwrap_or(ks.inkey).code())
        .filter(|code| Modifier::is_key_modifier(Key::from(*code)))
        .collect();

    mod_keys.sort();
    mod_keys
}

/// Get all pressed modifier objects from the current key states.
///
/// # Arguments
/// * `key_states` - Slice of all current keystates
///
/// # Returns
/// A vector of Modifier objects representing all pressed modifiers
pub fn get_pressed_mods(key_states: &[Keystate]) -> Vec<Modifier> {
    key_states
        .iter()
        .filter(|ks| ks.key_is_pressed())
        .map(|ks| ks.key.unwrap_or(ks.inkey))
        .filter(|key| Modifier::is_key_modifier(*key))
        .filter_map(|key| Modifier::from_key(key))
        .collect()
}

/// Get all pressed modifier keys from the current key states.
///
/// Returns the actual key codes (including remapped keys) for pressed modifiers.
/// This is preferred over `get_pressed_mods` when the specific key code matters
/// (e.g., after modmap remapping) rather than the modifier abstraction.
///
/// # Arguments
/// * `key_states` - Slice of all current keystates
///
/// # Returns
/// A vector of Key codes for pressed modifiers
pub fn get_pressed_mods_keys(key_states: &[Keystate]) -> Vec<Key> {
    key_states
        .iter()
        .filter(|ks| ks.key_is_pressed())
        .map(|ks| ks.key.unwrap_or(ks.inkey))
        .filter(|key| Modifier::is_key_modifier(*key))
        .collect()
}

/// Get all currently pressed keystates.
///
/// # Arguments
/// * `key_states` - Slice of all current keystates
///
/// # Returns
/// A vector of references to Keystate objects for pressed keys
pub fn get_pressed_states(key_states: &[Keystate]) -> Vec<&Keystate> {
    key_states.iter().filter(|ks| ks.key_is_pressed()).collect()
}

/// Get indices of pressed keystates that should be marked as spent.
///
/// This identifies pressed states whose keys are not in the provided set
/// of "pressed on output" keys. These states can have their spent flag set.
///
/// # Arguments
/// * `key_states` - Slice of all current keystates
/// * `pressed_output_keys` - Slice of Key codes that are pressed on output
///
/// # Returns
/// A vector of indices into key_states for states that should be marked spent
pub fn get_spent_state_indices(key_states: &[Keystate], pressed_output_keys: &[u16]) -> Vec<usize> {
    key_states
        .iter()
        .enumerate()
        .filter(|(_, ks)| {
            // Must be pressed and have a key
            ks.key_is_pressed()
                && ks.key.is_some()
                // Key must not be pressed on output
                && !pressed_output_keys.contains(&ks.key.unwrap().code())
        })
        .map(|(idx, _)| idx)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Action;

    #[test]
    fn test_get_modifier_snapshot_empty() {
        let states = vec![];
        let snapshot = get_modifier_snapshot(&states);
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_get_modifier_snapshot_with_modifiers() {
        let ctrl_state = Keystate::new(Key::from(29), Action::Press); // LEFT_CTRL
        let alt_state = Keystate::new(Key::from(56), Action::Press); // LEFT_ALT
        let a_state = Keystate::new(Key::from(30), Action::Press); // A (not a modifier)

        let states = vec![ctrl_state, alt_state, a_state];
        let snapshot = get_modifier_snapshot(&states);

        // Should contain only modifier codes (29, 56), sorted
        assert_eq!(snapshot.as_slice(), &[29, 56]);
    }

    #[test]
    fn test_get_pressed_mods() {
        let ctrl_state = Keystate::new(Key::from(29), Action::Press); // LEFT_CTRL
        let alt_state = Keystate::new(Key::from(56), Action::Press); // LEFT_ALT
        let a_state = Keystate::new(Key::from(30), Action::Press); // A (not a modifier)

        let states = vec![ctrl_state, alt_state, a_state];
        let mods = get_pressed_mods(&states);

        // Should get 2 modifiers
        assert_eq!(mods.len(), 2);
    }

    #[test]
    fn test_get_pressed_states() {
        let ctrl_state = Keystate::new(Key::from(29), Action::Press); // LEFT_CTRL (pressed)
        let alt_state = Keystate::new(Key::from(56), Action::Release); // LEFT_ALT (released)
        let a_state = Keystate::new(Key::from(30), Action::Press); // A (pressed)

        let states = vec![ctrl_state, alt_state, a_state];
        let pressed = get_pressed_states(&states);

        // Should only get pressed states
        assert_eq!(pressed.len(), 2);
    }

    #[test]
    fn test_get_spent_state_indices() {
        // Create keystates with keys set
        let mut ctrl_state = Keystate::new(Key::from(29), Action::Press);
        ctrl_state.key = Some(Key::from(29)); // LEFT_CTRL

        let mut alt_state = Keystate::new(Key::from(56), Action::Press);
        alt_state.key = Some(Key::from(56)); // LEFT_ALT

        let mut a_state = Keystate::new(Key::from(30), Action::Press);
        a_state.key = Some(Key::from(30)); // A

        let states = vec![ctrl_state, alt_state, a_state];

        // Case 1: No keys pressed on output - all should be spent
        let spent = get_spent_state_indices(&states, &[]);
        assert_eq!(spent.len(), 3);
        assert!(spent.contains(&0));
        assert!(spent.contains(&1));
        assert!(spent.contains(&2));

        // Case 2: Ctrl pressed on output - only Alt and A should be spent
        let spent = get_spent_state_indices(&states, &[29]);
        assert_eq!(spent.len(), 2);
        assert!(spent.contains(&1)); // Alt
        assert!(spent.contains(&2)); // A

        // Case 3: All pressed on output - none should be spent
        let spent = get_spent_state_indices(&states, &[29, 56, 30]);
        assert_eq!(spent.len(), 0);
    }

    #[test]
    fn test_get_pressed_mods_with_modmap() {
        // Create a keystate where inkey is LEFT_META but key is LEFT_CTRL (modmapped)
        let mut meta_state = Keystate::new(Key::from(125), Action::Press); // LEFT_META
        meta_state.key = Some(Key::from(29)); // Modmapped to LEFT_CTRL

        let alt_state = Keystate::new(Key::from(56), Action::Press); // LEFT_ALT

        let states = vec![meta_state, alt_state];
        let mods = get_pressed_mods(&states);

        // Should get LEFT_CTRL (remapped) not LEFT_META (original)
        assert_eq!(mods.len(), 2);

        // Get expected modifiers for comparison
        let left_ctrl = Modifier::from_key(Key::from(29)).unwrap();
        let left_alt = Modifier::from_key(Key::from(56)).unwrap();

        assert!(mods.iter().any(|m| m == &left_ctrl));
        assert!(mods.iter().any(|m| m == &left_alt));
    }

    #[test]
    fn test_get_modifier_snapshot_with_modmap() {
        let mut meta_state = Keystate::new(Key::from(125), Action::Press); // LEFT_META
        meta_state.key = Some(Key::from(29)); // Modmapped to LEFT_CTRL

        let states = vec![meta_state];
        let snapshot = get_modifier_snapshot(&states);

        // Should contain LEFT_CTRL code (29), not LEFT_META code (125)
        assert_eq!(snapshot.as_slice(), &[29]);
    }

    #[test]
    fn test_get_pressed_mods_keys() {
        let ctrl_state = Keystate::new(Key::from(29), Action::Press); // LEFT_CTRL
        let alt_state = Keystate::new(Key::from(56), Action::Press); // LEFT_ALT
        let a_state = Keystate::new(Key::from(30), Action::Press); // A (not a modifier)

        let states = vec![ctrl_state, alt_state, a_state];
        let keys = get_pressed_mods_keys(&states);

        // Should get 2 modifier keys (29, 56)
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&Key::from(29)));
        assert!(keys.contains(&Key::from(56)));
    }

    #[test]
    fn test_get_pressed_mods_keys_with_modmap() {
        // Test that remapped keys are returned correctly
        // LEFT_META (125) is modmapped to RIGHT_CTRL (97)
        let mut meta_state = Keystate::new(Key::from(125), Action::Press); // LEFT_META
        meta_state.key = Some(Key::from(97)); // Modmapped to RIGHT_CTRL

        let alt_state = Keystate::new(Key::from(56), Action::Press); // LEFT_ALT

        let states = vec![meta_state, alt_state];
        let keys = get_pressed_mods_keys(&states);

        // Should get RIGHT_CTRL (97) not LEFT_META (125), plus LEFT_ALT (56)
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&Key::from(97))); // RIGHT_CTRL (remapped)
        assert!(keys.contains(&Key::from(56))); // LEFT_ALT
    }
}

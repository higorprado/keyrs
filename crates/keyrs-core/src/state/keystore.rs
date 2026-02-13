// Keyrs Keystore
// Rust-side keystate storage with efficient accessor methods

use smallvec::SmallVec;
use std::collections::HashMap;

use crate::Action;
use crate::Key;
use crate::Keystate;
use crate::Modifier;

/// Rust-side keystore for efficient keystate management
///
/// This structure stores all keystates in Rust for optimal performance. It provides O(1) lookup by key code
/// and efficient modifier state queries.
#[derive(Debug)]
pub struct Keystore {
    /// Map from inkey code to Keystate
    /// We use u16 (key code) instead of Key as the key for more efficient lookups
    states: HashMap<u16, Keystate>,
}

impl Keystore {
    /// Create a new empty keystore
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Get the number of keystates in the store
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Get a keystate by key code
    pub fn get(&self, key_code: u16) -> Option<&Keystate> {
        self.states.get(&key_code)
    }

    /// Get a mutable reference to a keystate by key code
    pub fn get_mut(&mut self, key_code: u16) -> Option<&mut Keystate> {
        self.states.get_mut(&key_code)
    }

    /// Update or insert a keystate
    ///
    /// If a keystate with the same inkey exists, it will be replaced.
    /// The new keystate is created with a snapshot of the prior state if it exists.
    pub fn update(&mut self, inkey: Key, action: Action, key: Option<Key>) {
        let key_code = inkey.code();

        // Create new keystate, potentially with prior state snapshot
        let new_keystate = if let Some(prior) = self.states.get(&key_code) {
            Keystate::new(inkey, action).with_prior(prior.clone())
        } else {
            Keystate::new(inkey, action)
        };

        // Set the key if provided
        let mut new_keystate = new_keystate;
        if let Some(k) = key {
            new_keystate.key = Some(k);
        }

        self.states.insert(key_code, new_keystate);
    }

    /// Remove a keystate by key code
    pub fn remove(&mut self, key_code: u16) -> Option<Keystate> {
        self.states.remove(&key_code)
    }

    /// Clear all keystates
    pub fn clear(&mut self) {
        self.states.clear();
    }

    /// Get all keystates as a slice
    ///
    /// This is used for compatibility with existing transform utility functions.
    /// The returned Vec is allocated on each call, so this should be used sparingly.
    pub fn all_states(&self) -> Vec<Keystate> {
        self.states.values().cloned().collect()
    }

    /// Get current modifier state as a sorted vector of Key codes
    ///
    /// This returns a hashable representation for cache comparison.
    /// Returns pressed modifier Key codes, sorted by value for consistent comparison.
    /// Uses SmallVec to avoid heap allocation for the common case of 0-4 modifiers.
    pub fn get_modifier_snapshot(&self) -> SmallVec<[u16; 4]> {
        let mut mod_keys: SmallVec<[u16; 4]> = self
            .states
            .values()
            .filter(|ks| ks.key_is_pressed())
            .map(|ks| ks.key.unwrap_or(ks.inkey).code())
            .filter(|code| Modifier::is_key_modifier(Key::from(*code)))
            .collect();

        mod_keys.sort();
        mod_keys
    }

    /// Get all pressed modifier keys from the current key states
    ///
    /// Returns the actual key codes (including remapped keys) for pressed modifiers.
    pub fn get_pressed_mods_keys(&self) -> Vec<Key> {
        self.states
            .values()
            .filter(|ks| ks.key_is_pressed())
            .map(|ks| ks.key.unwrap_or(ks.inkey))
            .filter(|key| Modifier::is_key_modifier(*key))
            .collect()
    }

    /// Get all pressed modifier objects from the current key states
    ///
    /// Returns a vector of Modifier objects representing all pressed modifiers
    pub fn get_pressed_mods(&self) -> Vec<Modifier> {
        self.states
            .values()
            .filter(|ks| ks.key_is_pressed())
            .map(|ks| ks.key.unwrap_or(ks.inkey))
            .filter(|key| Modifier::is_key_modifier(*key))
            .filter_map(|key| Modifier::from_key(key))
            .collect()
    }

    /// Get all currently pressed keystates
    ///
    /// Returns a vector of Keystate objects for pressed keys
    pub fn get_pressed_states(&self) -> Vec<Keystate> {
        self.states
            .values()
            .filter(|ks| ks.key_is_pressed())
            .cloned()
            .collect()
    }

    /// Get indices of pressed keystates that should be marked as spent
    ///
    /// This identifies pressed states whose keys are not in the provided set
    /// of "pressed on output" keys. These states can have their spent flag set.
    ///
    /// Returns a vector of key codes for states that should be marked spent
    pub fn get_spent_state_keys(&self, pressed_output_keys: &[u16]) -> Vec<u16> {
        self.states
            .values()
            .filter(|ks| {
                // Must be pressed and have a key
                ks.key_is_pressed()
                    && ks.key.is_some()
                    // Key must not be pressed on output
                    && !pressed_output_keys.contains(&ks.key.unwrap().code())
            })
            .map(|ks| ks.inkey.code())
            .collect()
    }

    /// Iterate over all keystates
    pub fn iter(&self) -> impl Iterator<Item = &Keystate> {
        self.states.values()
    }

    /// Iterate over all keystates mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Keystate> {
        self.states.values_mut()
    }
}

impl Default for Keystore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_new() {
        let store = Keystore::new();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_keystore_update() {
        let mut store = Keystore::new();
        store.update(Key::from(30), Action::Press, None);
        assert_eq!(store.len(), 1);

        // Check the keystate
        let ks = store.get(30);
        assert!(ks.is_some());
        assert_eq!(ks.unwrap().inkey.code(), 30);
    }

    #[test]
    fn test_keystore_update_with_key() {
        let mut store = Keystore::new();
        store.update(Key::from(30), Action::Press, Some(Key::from(31)));

        let ks = store.get(30);
        assert!(ks.is_some());
        assert_eq!(ks.unwrap().key.unwrap().code(), 31);
    }

    #[test]
    fn test_keystore_remove() {
        let mut store = Keystore::new();
        store.update(Key::from(30), Action::Press, None);
        assert_eq!(store.len(), 1);

        store.remove(30);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_keystore_clear() {
        let mut store = Keystore::new();
        store.update(Key::from(30), Action::Press, None);
        store.update(Key::from(31), Action::Press, None);
        assert_eq!(store.len(), 2);

        store.clear();
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_get_modifier_snapshot_empty() {
        let store = Keystore::new();
        let snapshot = store.get_modifier_snapshot();
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_get_modifier_snapshot_with_modifiers() {
        let mut store = Keystore::new();
        store.update(Key::from(29), Action::Press, None); // LEFT_CTRL
        store.update(Key::from(56), Action::Press, None); // LEFT_ALT
        store.update(Key::from(30), Action::Press, None); // A (not a modifier)

        let snapshot = store.get_modifier_snapshot();

        // Should contain only modifier codes (29, 56), sorted
        assert_eq!(snapshot.as_slice(), &[29, 56]);
    }

    #[test]
    fn test_get_pressed_mods() {
        let mut store = Keystore::new();
        store.update(Key::from(29), Action::Press, None); // LEFT_CTRL
        store.update(Key::from(56), Action::Press, None); // LEFT_ALT
        store.update(Key::from(30), Action::Press, None); // A (not a modifier)

        let mods = store.get_pressed_mods();

        // Should get 2 modifiers
        assert_eq!(mods.len(), 2);
    }

    #[test]
    fn test_get_pressed_states() {
        let mut store = Keystore::new();
        store.update(Key::from(29), Action::Press, None); // LEFT_CTRL (pressed)
        store.update(Key::from(56), Action::Release, None); // LEFT_ALT (released)
        store.update(Key::from(30), Action::Press, None); // A (pressed)

        let pressed = store.get_pressed_states();

        // Should only get pressed states
        assert_eq!(pressed.len(), 2);
    }

    #[test]
    fn test_keystore_update_replaces() {
        let mut store = Keystore::new();

        // First update
        store.update(Key::from(30), Action::Press, None);
        let ks1 = store.get(30).unwrap();
        assert!(ks1.key_is_pressed()); // Action is Press, so should be pressed

        // Second update (release)
        store.update(Key::from(30), Action::Release, None);
        let ks2 = store.get(30).unwrap();
        assert!(!ks2.key_is_pressed()); // Action is Release

        // Should still be only one entry
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_keystore_get_pressed_mods_keys() {
        let mut store = Keystore::new();
        store.update(Key::from(29), Action::Press, None); // LEFT_CTRL
        store.update(Key::from(56), Action::Press, None); // LEFT_ALT
        store.update(Key::from(30), Action::Press, None); // A (not a modifier)

        let keys = store.get_pressed_mods_keys();

        // Should get 2 modifier keys (29, 56)
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&Key::from(29)));
        assert!(keys.contains(&Key::from(56)));
    }

    #[test]
    fn test_keystore_get_pressed_mods_keys_with_modmap() {
        let mut store = Keystore::new();
        // LEFT_META (125) is modmapped to RIGHT_CTRL (97)
        store.update(Key::from(125), Action::Press, Some(Key::from(97)));
        store.update(Key::from(56), Action::Press, None); // LEFT_ALT

        let keys = store.get_pressed_mods_keys();

        // Should get RIGHT_CTRL (97) not LEFT_META (125), plus LEFT_ALT (56)
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&Key::from(97))); // RIGHT_CTRL (remapped)
        assert!(keys.contains(&Key::from(56))); // LEFT_ALT
    }

    #[test]
    fn test_get_spent_state_keys() {
        let mut store = Keystore::new();
        store.update(Key::from(29), Action::Press, Some(Key::from(29))); // LEFT_CTRL
        store.update(Key::from(56), Action::Press, Some(Key::from(56))); // LEFT_ALT
        store.update(Key::from(30), Action::Press, Some(Key::from(30))); // A

        // Case 1: No keys pressed on output - all should be spent
        let spent = store.get_spent_state_keys(&[]);
        assert_eq!(spent.len(), 3);

        // Case 2: Ctrl pressed on output - only Alt and A should be spent
        let spent = store.get_spent_state_keys(&[29]);
        assert_eq!(spent.len(), 2);

        // Case 3: All pressed on output - none should be spent
        let spent = store.get_spent_state_keys(&[29, 56, 30]);
        assert_eq!(spent.len(), 0);
    }
}

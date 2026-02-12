// Xwaykeyz Pressed Key State Management
// HashSet-based O(1) key lookup for pressed keys

use crate::Key;
use std::collections::HashSet;

/// Tracks pressed keys with O(1) lookup performance
#[derive(Debug, Clone)]
pub struct PressedKeyState {
    pressed: HashSet<Key>,
}

impl Default for PressedKeyState {
    fn default() -> Self {
        Self::new()
    }
}

impl PressedKeyState {
    /// Create a new empty pressed key state
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }

    /// Add a key to the pressed state
    pub fn add(&mut self, key: Key) {
        self.pressed.insert(key);
    }

    /// Remove a key from the pressed state
    pub fn remove(&mut self, key: Key) {
        self.pressed.remove(&key);
    }

    /// Check if a key is currently pressed
    pub fn is_pressed(&self, key: Key) -> bool {
        self.pressed.contains(&key)
    }

    /// Add a key by code directly (avoids Key allocation)
    pub fn add_code(&mut self, code: u16) {
        self.pressed.insert(Key::from(code));
    }

    /// Remove a key by code directly (avoids Key allocation)
    pub fn remove_code(&mut self, code: u16) {
        self.pressed.remove(&Key::from(code));
    }

    /// Check if a key code is currently pressed (avoids Key allocation)
    pub fn is_pressed_code(&self, code: u16) -> bool {
        self.pressed.contains(&Key::from(code))
    }

    /// Get all pressed keys
    pub fn get_all(&self) -> Vec<Key> {
        self.pressed.iter().copied().collect()
    }

    /// Clear all pressed keys
    pub fn clear(&mut self) {
        self.pressed.clear();
    }

    /// Get the number of pressed keys
    pub fn len(&self) -> usize {
        self.pressed.len()
    }

    /// Check if the state is empty
    pub fn is_empty(&self) -> bool {
        self.pressed.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_add_remove() {
        let mut state = PressedKeyState::new();
        let key = Key::from(30); // A

        assert!(!state.is_pressed(key));
        state.add(key);
        assert!(state.is_pressed(key));
        state.remove(key);
        assert!(!state.is_pressed(key));
    }

    #[test]
    fn test_state_is_pressed() {
        let mut state = PressedKeyState::new();
        let key_a = Key::from(30);
        let key_b = Key::from(48); // B

        state.add(key_a);
        assert!(state.is_pressed(key_a));
        assert!(!state.is_pressed(key_b));
    }

    #[test]
    fn test_state_get_all() {
        let mut state = PressedKeyState::new();
        let key_a = Key::from(30);
        let key_b = Key::from(48);
        let key_c = Key::from(46); // C

        state.add(key_a);
        state.add(key_b);
        state.add(key_c);

        let all = state.get_all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&key_a));
        assert!(all.contains(&key_b));
        assert!(all.contains(&key_c));
    }

    #[test]
    fn test_state_clear() {
        let mut state = PressedKeyState::new();
        state.add(Key::from(30));
        state.add(Key::from(48));

        assert_eq!(state.len(), 2);
        state.clear();
        assert_eq!(state.len(), 0);
        assert!(state.is_empty());
    }

    #[test]
    fn test_state_duplicate_add() {
        let mut state = PressedKeyState::new();
        let key = Key::from(30);

        state.add(key);
        state.add(key); // Adding same key twice should be idempotent

        assert_eq!(state.len(), 1);
        assert!(state.is_pressed(key));
    }

    #[test]
    fn test_state_remove_nonexistent() {
        let mut state = PressedKeyState::new();
        let key = Key::from(30);

        // Removing a non-existent key should not panic
        state.remove(key);
        assert!(state.is_empty());
    }

    #[test]
    fn test_state_default() {
        let state = PressedKeyState::default();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_state_add_code() {
        let mut state = PressedKeyState::new();
        state.add_code(30); // A
        assert!(state.is_pressed_code(30));
        assert!(!state.is_pressed_code(48)); // B
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_state_remove_code() {
        let mut state = PressedKeyState::new();
        state.add_code(30);
        state.add_code(48);
        assert_eq!(state.len(), 2);

        state.remove_code(30);
        assert!(!state.is_pressed_code(30));
        assert!(state.is_pressed_code(48));
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_state_is_pressed_code() {
        let mut state = PressedKeyState::new();
        assert!(!state.is_pressed_code(30));

        state.add_code(30);
        assert!(state.is_pressed_code(30));

        state.remove_code(30);
        assert!(!state.is_pressed_code(30));
    }
}

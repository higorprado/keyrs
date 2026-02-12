// Xwaykeyz Combo Cache
// Pre-computed HashMap for O(1) combo lookup instead of O(n) iteration

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::mapping::KeymapValue;
use crate::{Combo, Key, Modifier};

/// A hashable key for combo lookups
///
/// We need a custom hashable representation because:
/// 1. The combo itself has non-hashable Vec fields
/// 2. We want to sort modifiers for consistent hashing
/// 3. We only care about modifier key codes, not Modifier objects
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComboKey {
    /// Sorted modifier key codes (for consistent hashing)
    pub modifier_codes: Vec<u16>,
    /// The main key code
    pub key_code: u16,
}

impl ComboKey {
    /// Create a new ComboKey from modifiers and a key
    pub fn new(modifiers: &[Modifier], key: Key) -> Self {
        let mut modifier_codes: Vec<u16> = modifiers
            .iter()
            .flat_map(|m| m.keys())
            .map(|k| k.code())
            .collect();

        // Sort for consistent hashing (Ctrl-A vs A-Ctrl should match)
        modifier_codes.sort();
        modifier_codes.dedup(); // Remove duplicates

        Self {
            modifier_codes,
            key_code: key.code(),
        }
    }

    /// Create from a Combo object
    pub fn from_combo(combo: &Combo) -> Self {
        Self::new(combo.modifiers(), combo.key())
    }
}

impl Hash for ComboKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the modifier codes and key code
        self.modifier_codes.hash(state);
        self.key_code.hash(state);
    }
}

/// Pre-computed combo cache for O(1) lookups
///
/// Instead of iterating through all keymaps on every keypress,
/// we build a HashMap once at config load time for constant-time lookup.
#[derive(Debug, Clone)]
pub struct KeymapCache {
    /// The cache: combo key -> output value
    cache: HashMap<ComboKey, KeymapValue>,
}

impl KeymapCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Build a cache from a list of keymaps
    ///
    /// This should be called once at startup after parsing configuration.
    /// It iterates through all keymaps and builds a unified HashMap.
    pub fn build(keymaps: &[crate::mapping::Keymap]) -> Self {
        let mut cache = HashMap::new();

        for keymap in keymaps {
            // Iterate through all mappings in this keymap
            for (combo, value) in keymap.mappings() {
                let key = ComboKey::from_combo(combo);
                // Later keymaps override earlier ones (same as iteration order)
                cache.insert(key, value.clone());
            }
        }

        Self { cache }
    }

    /// Look up a combo in the cache (O(1))
    ///
    /// Returns None if the combo is not found
    pub fn lookup(&self, modifiers: &[Modifier], key: Key) -> Option<&KeymapValue> {
        let combo_key = ComboKey::new(modifiers, key);
        self.cache.get(&combo_key)
    }

    /// Look up a combo from a Combo object
    pub fn lookup_combo(&self, combo: &Combo) -> Option<&KeymapValue> {
        let combo_key = ComboKey::from_combo(combo);
        self.cache.get(&combo_key)
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get the number of cached combos
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for KeymapCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::Keymap;

    #[test]
    fn test_combo_key_from_combo() {
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::new(vec![ctrl], Key::from(30)); // Ctrl-A
        let key = ComboKey::from_combo(&combo);

        assert!(!key.modifier_codes.is_empty());
        assert_eq!(key.key_code, 30);
    }

    #[test]
    fn test_combo_key_sorted_modifiers() {
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let shift = Modifier::from_alias("Shift").unwrap();
        let combo = Combo::new(vec![shift, ctrl], Key::from(30)); // Shift-Ctrl-A
        let key = ComboKey::from_combo(&combo);

        // Modifiers should be sorted
        let mut sorted = key.modifier_codes.clone();
        sorted.sort();
        assert_eq!(key.modifier_codes, sorted);
    }

    #[test]
    fn test_keymap_cache_build() {
        let mut keymap = Keymap::new("test");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::new(vec![ctrl.clone()], Key::from(30)); // Ctrl-A
        keymap.insert(combo, KeymapValue::Key(Key::from(31))); // -> S

        let cache = KeymapCache::build(&[keymap]);

        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_keymap_cache_lookup() {
        let mut keymap = Keymap::new("test");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::new(vec![ctrl.clone()], Key::from(30)); // Ctrl-A
        keymap.insert(combo, KeymapValue::Key(Key::from(31))); // -> S

        let cache = KeymapCache::build(&[keymap]);

        let result = cache.lookup(&[ctrl], Key::from(30));
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &KeymapValue::Key(Key::from(31)));
    }

    #[test]
    fn test_keymap_cache_lookup_not_found() {
        let keymap = Keymap::new("test");
        let cache = KeymapCache::build(&[keymap]);

        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let result = cache.lookup(&[ctrl], Key::from(30));
        assert!(result.is_none());
    }

    #[test]
    fn test_keymap_cache_override() {
        // Two keymaps with the same combo - later should override
        let mut keymap1 = Keymap::new("first");
        let mut keymap2 = Keymap::new("second");

        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::new(vec![ctrl.clone()], Key::from(30)); // Ctrl-A

        keymap1.insert(combo.clone(), KeymapValue::Key(Key::from(31))); // -> S
        keymap2.insert(combo, KeymapValue::Key(Key::from(32))); // -> D

        let cache = KeymapCache::build(&[keymap1, keymap2]);

        let result = cache.lookup(&[ctrl], Key::from(30));
        assert_eq!(result.unwrap(), &KeymapValue::Key(Key::from(32))); // Should get D (second)
    }
}

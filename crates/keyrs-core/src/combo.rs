// Xwaykeyz Combo Type
// Represents a key combination with modifiers

use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::modifier::Modifier;
use crate::Key;

/// Special combo hints for keymap behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ComboHint {
    Bind = 1,
    EscapeNextKey = 2,
    Ignore = 3,
    EscapeNextCombo = 4,
}

impl ComboHint {
    /// Create ComboHint from i32 value
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(ComboHint::Bind),
            2 => Some(ComboHint::EscapeNextKey),
            3 => Some(ComboHint::Ignore),
            4 => Some(ComboHint::EscapeNextCombo),
            _ => None,
        }
    }

    /// Convert to i32
    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for ComboHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComboHint::Bind => write!(f, "BIND"),
            ComboHint::EscapeNextKey => write!(f, "ESCAPE_NEXT_KEY"),
            ComboHint::Ignore => write!(f, "IGNORE"),
            ComboHint::EscapeNextCombo => write!(f, "ESCAPE_NEXT_COMBO"),
        }
    }
}

/// Represents a key combination with an ordered set of modifiers
#[derive(Debug, Clone)]
pub struct Combo {
    modifiers: Vec<Modifier>,
    key: Key,
}

impl Combo {
    /// Create a new Combo from modifiers and a key
    ///
    /// # Arguments
    /// * `modifiers` - Iterator of modifiers
    /// * `key` - The key code
    pub fn new(modifiers: impl IntoIterator<Item = Modifier>, key: Key) -> Self {
        Self {
            modifiers: modifiers.into_iter().collect(),
            key,
        }
    }

    /// Create a Combo from a single modifier and key
    pub fn from_single(modifier: Modifier, key: Key) -> Self {
        Self {
            modifiers: vec![modifier],
            key,
        }
    }

    /// Get the modifiers for this combo
    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    /// Get the key for this combo
    pub fn key(&self) -> Key {
        self.key
    }

    /// Add a modifier to this combo
    pub fn with_modifier(&self, modifier: Modifier) -> Self {
        let mut new_modifiers = self.modifiers.clone();
        new_modifiers.push(modifier);
        Self {
            modifiers: new_modifiers,
            key: self.key,
        }
    }

    /// Add modifiers to this combo
    pub fn with_modifiers(&self, modifiers: impl IntoIterator<Item = Modifier>) -> Self {
        let mut new_modifiers = self.modifiers.clone();
        new_modifiers.extend(modifiers);
        Self {
            modifiers: new_modifiers,
            key: self.key,
        }
    }
}

impl PartialEq for Combo {
    fn eq(&self, other: &Self) -> bool {
        // Order-independent equality: compare sets of modifiers
        let self_modifiers: HashSet<_> = self.modifiers.iter().collect();
        let other_modifiers: HashSet<_> = other.modifiers.iter().collect();
        self_modifiers == other_modifiers && self.key == other.key
    }
}

impl Eq for Combo {}

impl Hash for Combo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash based on frozenset of modifiers (order-independent)
        let modifier_hash: u64 = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            let mut modifier_ids: Vec<u64> = self
                .modifiers
                .iter()
                .map(|m| {
                    // Use the Modifier's Hash implementation
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    m.hash(&mut h);
                    h.finish()
                })
                .collect();
            modifier_ids.sort(); // Sort for order-independent hashing
            for id in modifier_ids {
                id.hash(&mut hasher);
            }
            hasher.finish()
        };
        modifier_hash.hash(state);
        self.key.hash(state);
    }
}

impl fmt::Display for Combo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.modifiers.iter().map(|m| m.to_string()).collect();
        write!(f, "{}-{}", parts.join("-"), self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifier::Modifier;

    #[test]
    fn test_combo_equality_order_independent() {
        let ctrl_a = Combo::from_single(Modifier::from_alias("Ctrl").unwrap(), Key::from(30)); // A
        let ctrl_a2 = Combo::new(vec![Modifier::from_alias("Ctrl").unwrap()], Key::from(30));
        assert_eq!(ctrl_a, ctrl_a2);
    }

    #[test]
    fn test_combo_display() {
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::from_single(ctrl, Key::from(30)); // A
        let s = combo.to_string();
        assert!(s.contains("Ctrl"));
        assert!(s.contains("A"));
    }

    #[test]
    fn test_combo_with_modifier() {
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let shift = Modifier::from_alias("Shift").unwrap();
        let combo = Combo::from_single(ctrl, Key::from(30)); // A
        let combo_with_shift = combo.with_modifier(shift);
        assert_eq!(combo_with_shift.modifiers().len(), 2);
    }

    #[test]
    fn test_combo_hashable() {
        use std::collections::HashMap;
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo1 = Combo::from_single(ctrl.clone(), Key::from(30)); // A
        let combo2 = Combo::from_single(ctrl, Key::from(30)); // A

        let mut map: HashMap<Combo, String> = HashMap::new();
        map.insert(combo1, "value".to_string());
        assert_eq!(map.get(&combo2), Some(&"value".to_string()));
    }

    #[test]
    fn test_combo_hint_from_i32() {
        assert_eq!(ComboHint::from_i32(1), Some(ComboHint::Bind));
        assert_eq!(ComboHint::from_i32(2), Some(ComboHint::EscapeNextKey));
        assert_eq!(ComboHint::from_i32(5), None);
    }
}

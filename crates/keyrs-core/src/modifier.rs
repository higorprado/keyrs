// Xwaykeyz Modifier System
// Represents keyboard combo modifiers (Shift, Ctrl, Alt, Meta)

use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::Key;

/// Global modifier registry
static MODIFIER_REGISTRY: LazyLock<RwLock<ModifierRegistry>> = LazyLock::new(|| {
    // Initialize default modifiers on first access
    RwLock::new(ModifierRegistry::with_defaults())
});
static NEXT_MODIFIER_ID: AtomicU32 = AtomicU32::new(100);

use std::sync::LazyLock;

/// Internal registry for modifiers
struct ModifierRegistry {
    by_name: HashMap<String, Modifier>,
    by_key: HashMap<Key, Modifier>,
    by_alias: HashMap<String, Modifier>,
}

impl ModifierRegistry {
    fn empty() -> Self {
        Self {
            by_name: HashMap::new(),
            by_key: HashMap::new(),
            by_alias: HashMap::new(),
        }
    }

    fn with_defaults() -> Self {
        let mut registry = Self::empty();
        // Add default modifiers
        registry.add_internal("R_CONTROL", &["RCtrl", "RC"], vec![Key::from(97)]);
        registry.add_internal("L_CONTROL", &["LCtrl", "LC"], vec![Key::from(29)]);
        registry.add_internal(
            "CONTROL",
            &["Ctrl", "C"],
            vec![Key::from(29), Key::from(97)],
        );
        registry.add_internal(
            "R_ALT",
            &["RAlt", "RA", "ROpt", "ROption"],
            vec![Key::from(100)],
        );
        registry.add_internal(
            "L_ALT",
            &["LAlt", "LA", "LOpt", "LOption"],
            vec![Key::from(56)],
        );
        registry.add_internal(
            "ALT",
            &["Alt", "A", "Opt", "Option"],
            vec![Key::from(56), Key::from(100)],
        );
        registry.add_internal("R_SHIFT", &["RShift"], vec![Key::from(54)]);
        registry.add_internal("L_SHIFT", &["LShift"], vec![Key::from(42)]);
        registry.add_internal("SHIFT", &["Shift"], vec![Key::from(42), Key::from(54)]);
        registry.add_internal(
            "R_META",
            &["RSuper", "RWin", "RCommand", "RCmd", "RMeta"],
            vec![Key::from(126)],
        );
        registry.add_internal(
            "L_META",
            &["LSuper", "LWin", "LCommand", "LCmd", "LMeta"],
            vec![Key::from(125)],
        );
        registry.add_internal(
            "META",
            &["Super", "Win", "Command", "Cmd", "Meta"],
            vec![Key::from(125), Key::from(126)],
        );
        registry.add_internal("FN", &["Fn"], vec![Key::from(0x1d0)]);
        registry
    }

    fn add_internal(&mut self, name: &str, aliases: &[&str], keys: Vec<Key>) {
        let modifier = Modifier {
            id: NEXT_MODIFIER_ID.fetch_add(1, Ordering::SeqCst),
            name: name.to_string(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            keys,
        };

        // Register by name
        self.by_name.insert(name.to_string(), modifier.clone());

        // Register aliases
        for alias in aliases {
            self.by_alias.insert(alias.to_string(), modifier.clone());
        }

        // Register keys
        for &key in &modifier.keys {
            self.by_key.insert(key, modifier.clone());
        }
    }

    fn add(&mut self, modifier: Modifier) -> Result<(), ModifierError> {
        // Check if name already exists
        if self.by_name.contains_key(&modifier.name) {
            return Err(ModifierError::NameExists(modifier.name.clone()));
        }

        // Register keys
        for &key in &modifier.keys {
            if let Some(existing) = self.by_key.get(&key) {
                return Err(ModifierError::KeyAlreadyAssigned(
                    key,
                    existing.name.clone(),
                ));
            }
            self.by_key.insert(key, modifier.clone());
        }

        // Register aliases
        for alias in &modifier.aliases {
            if alias != &modifier.name {
                self.by_alias.insert(alias.clone(), modifier.clone());
            }
        }

        self.by_name.insert(modifier.name.clone(), modifier);
        Ok(())
    }
}

/// Represents a keyboard combo modifier, such as Shift or Cmd
#[derive(Debug, Clone)]
pub struct Modifier {
    id: u32,
    name: String,
    aliases: Vec<String>,
    keys: Vec<Key>,
}

impl Modifier {
    /// Add a new modifier to the global registry
    ///
    /// # Arguments
    /// * `name` - Unique name for the modifier
    /// * `aliases` - List of alias strings
    /// * `keys` - List of keys that represent this modifier
    pub fn add(name: &str, aliases: Vec<String>, keys: Vec<Key>) -> Result<(), ModifierError> {
        let modifier = Modifier {
            id: NEXT_MODIFIER_ID.fetch_add(1, Ordering::SeqCst),
            name: name.to_string(),
            aliases,
            keys,
        };
        MODIFIER_REGISTRY.write().add(modifier)
    }

    /// Get the first alias (string representation)
    pub fn primary_alias(&self) -> &str {
        self.aliases
            .first()
            .map(|s| s.as_str())
            .unwrap_or(&self.name)
    }

    /// Check if this is a specific modifier (single key)
    pub fn is_specific(&self) -> bool {
        self.keys.len() == 1
    }

    /// Get all keys for this modifier
    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    /// Get the first key (panics if no keys)
    pub fn key(&self) -> Key {
        self.keys[0]
    }

    /// Get the left variant of this generic modifier
    pub fn to_left(&self) -> Option<Modifier> {
        if self.name.starts_with("L_") {
            return Some(self.clone());
        }
        let left_name = format!("L_{}", self.name);
        MODIFIER_REGISTRY.read().by_name.get(&left_name).cloned()
    }

    /// Get the right variant of this generic modifier
    pub fn to_right(&self) -> Option<Modifier> {
        if self.name.starts_with("R_") {
            return Some(self.clone());
        }
        let right_name = format!("R_{}", self.name);
        MODIFIER_REGISTRY.read().by_name.get(&right_name).cloned()
    }

    /// Get modifier by key code
    pub fn from_key(key: Key) -> Option<Modifier> {
        MODIFIER_REGISTRY.read().by_key.get(&key).cloned()
    }

    /// Check if a key is a modifier (fast path using static array)
    ///
    /// This uses the compile-time generated static array for O(1) lock-free lookup.
    /// For custom modifiers added at runtime, it falls back to the registry lookup.
    pub fn is_key_modifier(key: Key) -> bool {
        // Fast path: check static array first (O(1), lock-free)
        if is_key_modifier_code(key.code()) {
            return true;
        }
        // Slow path: check registry for custom modifiers
        MODIFIER_REGISTRY.read().by_key.contains_key(&key)
    }

    /// Get modifier by name
    pub fn from_name(name: &str) -> Option<Modifier> {
        MODIFIER_REGISTRY.read().by_name.get(name).cloned()
    }

    /// Get modifier by alias
    pub fn from_alias(alias: &str) -> Option<Modifier> {
        // Try direct name first
        if let Some(m) = Self::from_name(alias) {
            return Some(m);
        }
        // Try alias map
        MODIFIER_REGISTRY.read().by_alias.get(alias).cloned()
    }

    /// Get modifier name for a key
    pub fn key_name(key: Key) -> Option<String> {
        MODIFIER_REGISTRY
            .read()
            .by_key
            .get(&key)
            .map(|m| m.name.clone())
    }

    /// Get all aliases
    pub fn all_aliases() -> Vec<String> {
        let registry = MODIFIER_REGISTRY.read();
        let mut aliases = Vec::new();
        for modifier in registry.by_name.values() {
            for alias in &modifier.aliases {
                aliases.push(alias.clone());
            }
        }
        aliases
    }
}

impl PartialEq for Modifier {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Modifier {}

impl std::hash::Hash for Modifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.primary_alias())
    }
}

/// Errors that can occur when working with modifiers
#[derive(Debug, Clone, PartialEq)]
pub enum ModifierError {
    NameExists(String),
    KeyAlreadyAssigned(Key, String),
}

impl fmt::Display for ModifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModifierError::NameExists(name) => {
                write!(f, "modifier '{}' already exists", name)
            }
            ModifierError::KeyAlreadyAssigned(key, existing) => {
                write!(
                    f,
                    "key {:?} already assigned to modifier '{}'",
                    key, existing
                )
            }
        }
    }
}

impl std::error::Error for ModifierError {}

/// Static bitmask array for O(1) lock-free modifier lookup
///
/// This is a compile-time generated array that provides constant-time
/// checking of whether a key code is a modifier. This eliminates the
/// RwLock overhead from the global registry lookup on every keystroke.
const MODIFIER_KEY_CODES: &[u16] = &[
    29, 97, // L_CONTROL, R_CONTROL
    56, 100, // L_ALT, R_ALT
    42, 54, // L_SHIFT, R_SHIFT
    125, 126,   // L_META, R_META
    0x1d0, // FN
];

/// Check if a key code is a modifier using static array (O(1) lock-free)
///
/// This is the fast path that should be used in hot loops. It checks against
/// a compile-time generated array of all modifier key codes.
#[inline]
pub const fn is_key_modifier_code(code: u16) -> bool {
    let mut i = 0;
    while i < MODIFIER_KEY_CODES.len() {
        if MODIFIER_KEY_CODES[i] == code {
            return true;
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_from_key() {
        let ctrl = Modifier::from_key(Key::from(29)); // LEFT_CTRL
                                                      // Can be either L_CONTROL or CONTROL since both use this key
        assert!(ctrl.is_some());
        let name = ctrl.unwrap().name;
        assert!(name == "L_CONTROL" || name == "CONTROL");
    }

    #[test]
    fn test_modifier_from_alias() {
        let ctrl = Modifier::from_alias("Ctrl");
        assert_eq!(ctrl.unwrap().name, "CONTROL");
        let lctrl = Modifier::from_alias("LCtrl");
        assert_eq!(lctrl.unwrap().name, "L_CONTROL");
    }

    #[test]
    fn test_modifier_to_left_right() {
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let left = ctrl.to_left().unwrap();
        assert_eq!(left.name, "L_CONTROL");
        let right = ctrl.to_right().unwrap();
        assert_eq!(right.name, "R_CONTROL");
    }

    #[test]
    fn test_is_key_modifier() {
        assert!(Modifier::is_key_modifier(Key::from(29))); // LEFT_CTRL
        assert!(!Modifier::is_key_modifier(Key::from(30))); // A
    }

    #[test]
    fn test_is_key_modifier_code_static() {
        // Test the fast static path
        assert!(is_key_modifier_code(29)); // LEFT_CTRL
        assert!(is_key_modifier_code(97)); // RIGHT_CTRL
        assert!(is_key_modifier_code(56)); // LEFT_ALT
        assert!(is_key_modifier_code(100)); // RIGHT_ALT
        assert!(is_key_modifier_code(42)); // LEFT_SHIFT
        assert!(is_key_modifier_code(54)); // RIGHT_SHIFT
        assert!(is_key_modifier_code(125)); // LEFT_META
        assert!(is_key_modifier_code(126)); // RIGHT_META
        assert!(is_key_modifier_code(0x1d0)); // FN

        // Non-modifier keys
        assert!(!is_key_modifier_code(30)); // A
        assert!(!is_key_modifier_code(57)); // SPACE
    }
}

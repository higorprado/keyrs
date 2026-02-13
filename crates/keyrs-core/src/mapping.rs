// Keyrs Mapping Structures
// Modmap, MultiModmap, Keymap, Keystate

use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

use crate::Action;
use crate::Combo;
use crate::Key;

/// Simple key remapping (one key to another)
#[derive(Debug, Clone)]
pub struct Modmap {
    name: String,
    mappings: HashMap<Key, Key>,
    conditional: Option<String>,
}

impl Modmap {
    /// Create a new Modmap
    pub fn new(name: impl Into<String>, mappings: HashMap<Key, Key>) -> Self {
        Self {
            name: name.into(),
            mappings,
            conditional: None,
        }
    }

    /// Create a new Modmap with a conditional
    pub fn with_conditional(
        name: impl Into<String>,
        mappings: HashMap<Key, Key>,
        conditional: String,
    ) -> Self {
        Self {
            name: name.into(),
            mappings,
            conditional: Some(conditional),
        }
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the mappings
    pub fn mappings(&self) -> &HashMap<Key, Key> {
        &self.mappings
    }

    /// Get the conditional (if any)
    pub fn conditional(&self) -> Option<&str> {
        self.conditional.as_deref()
    }

    /// Check if a key is in this modmap
    pub fn contains(&self, key: Key) -> bool {
        self.mappings.contains_key(&key)
    }

    /// Get the remapped key for a given key
    pub fn get(&self, key: Key) -> Option<Key> {
        self.mappings.get(&key).copied()
    }
}

/// Multipurpose key mapping (tap vs. hold)
#[derive(Debug, Clone)]
pub struct MultiModmap {
    name: String,
    mappings: HashMap<Key, (Key, Key)>, // (tap_key, hold_key)
    conditional: Option<String>,
}

impl MultiModmap {
    /// Create a new MultiModmap
    pub fn new(name: impl Into<String>, mappings: HashMap<Key, (Key, Key)>) -> Self {
        Self {
            name: name.into(),
            mappings,
            conditional: None,
        }
    }

    /// Create a new MultiModmap with a conditional
    pub fn with_conditional(
        name: impl Into<String>,
        mappings: HashMap<Key, (Key, Key)>,
        conditional: String,
    ) -> Self {
        Self {
            name: name.into(),
            mappings,
            conditional: Some(conditional),
        }
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the mappings
    pub fn mappings(&self) -> &HashMap<Key, (Key, Key)> {
        &self.mappings
    }

    /// Get the conditional (if any)
    pub fn conditional(&self) -> Option<&str> {
        self.conditional.as_deref()
    }

    /// Check if a key is in this multi-modmap
    pub fn contains(&self, key: Key) -> bool {
        self.mappings.contains_key(&key)
    }

    /// Get the (tap, hold) keys for a given key
    pub fn get(&self, key: Key) -> Option<(Key, Key)> {
        self.mappings.get(&key).copied()
    }

    /// Iterate over all mappings
    pub fn iter(&self) -> impl Iterator<Item = (Key, Key, Key)> + '_ {
        self.mappings.iter().map(|(k, (t, h))| (*k, *t, *h))
    }
}

/// Keymap for key combinations
#[derive(Debug, Clone)]
pub struct Keymap {
    name: String,
    mappings: HashMap<Combo, KeymapValue>,
    conditional: Option<String>,
}

/// Value in a keymap - can be a Combo, ComboHint, or a key
#[derive(Debug, Clone, PartialEq)]
pub enum KeymapValue {
    Combo(Combo),
    Sequence(Vec<ActionStep>),
    ComboHint(ComboHint),
    Key(Key),
    Unicode(u32),
    Text(String),
}

/// A single step in a keymap output sequence.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionStep {
    Combo(Combo),
    Text(String),
    DelayMs(u64),
    Ignore,
    Bind,
    SetSetting { name: String, value: bool },
}

impl From<Combo> for KeymapValue {
    fn from(combo: Combo) -> Self {
        KeymapValue::Combo(combo)
    }
}

impl From<ComboHint> for KeymapValue {
    fn from(hint: ComboHint) -> Self {
        KeymapValue::ComboHint(hint)
    }
}

impl From<Key> for KeymapValue {
    fn from(key: Key) -> Self {
        KeymapValue::Key(key)
    }
}

impl From<u32> for KeymapValue {
    fn from(codepoint: u32) -> Self {
        KeymapValue::Unicode(codepoint)
    }
}

use crate::ComboHint;

impl Keymap {
    /// Create a new Keymap
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mappings: HashMap::new(),
            conditional: None,
        }
    }

    /// Create a new Keymap with mappings
    pub fn with_mappings(name: impl Into<String>, mappings: HashMap<Combo, KeymapValue>) -> Self {
        Self {
            name: name.into(),
            mappings,
            conditional: None,
        }
    }

    /// Create a new Keymap with a conditional
    pub fn with_conditional(name: impl Into<String>, mappings: HashMap<Combo, KeymapValue>, conditional: String) -> Self {
        Self {
            name: name.into(),
            mappings,
            conditional: Some(conditional),
        }
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the mappings
    pub fn mappings(&self) -> &HashMap<Combo, KeymapValue> {
        &self.mappings
    }

    /// Get the conditional (if any)
    pub fn conditional(&self) -> Option<&str> {
        self.conditional.as_deref()
    }

    /// Check if a combo is in this keymap
    pub fn contains(&self, combo: &Combo) -> bool {
        self.mappings.contains_key(combo)
    }

    /// Get the value for a given combo
    pub fn get(&self, combo: &Combo) -> Option<&KeymapValue> {
        self.mappings.get(combo)
    }

    /// Insert a mapping
    pub fn insert(&mut self, combo: Combo, value: KeymapValue) {
        self.mappings.insert(combo, value);
    }
}

/// State of a key during processing
#[derive(Debug, Clone)]
pub struct Keystate {
    /// The actual REAL key pressed on input device
    pub inkey: Key,
    /// Current action state: PRESS, REPEAT, or RELEASE
    pub action: Action,
    /// Copy of previous keystate, for tracking state changes
    pub prior: Option<Box<Keystate>>,
    /// Timestamp when keystate was created or updated
    pub time: Instant,
    /// The key we modmapped to (may differ from inkey)
    pub key: Option<Key>,
    /// The modifier we may modmap to (multi-key) if used
    /// as part of a combo or held for a certain time period
    pub multikey: Option<Key>,
    /// Whether this key is currently suspended inside the
    /// transform engine waiting for other input
    pub suspended: bool,
    /// Whether this key is a multipurpose key (tap vs hold behavior)
    pub is_multi: bool,
    /// Whether this key's press has been sent to output device
    pub exerted_on_output: bool,
    /// If this keystate was spent by executing a combo
    pub spent: bool,
    /// Track if any other key was pressed while this multikey was held
    pub other_key_pressed_while_held: bool,
}

impl Keystate {
    /// Create a new Keystate
    pub fn new(inkey: Key, action: Action) -> Self {
        Self {
            inkey,
            action,
            prior: None,
            time: Instant::now(),
            key: None,
            multikey: None,
            suspended: false,
            is_multi: false,
            exerted_on_output: false,
            spent: false,
            other_key_pressed_while_held: false,
        }
    }

    /// Create a new Keystate with a snapshot of the current state
    pub fn with_prior(mut self, prior: Keystate) -> Self {
        self.prior = Some(Box::new(prior));
        self
    }

    /// Check if the key is pressed (PRESS or REPEAT)
    pub fn key_is_pressed(&self) -> bool {
        self.action.is_pressed()
    }

    /// Resolve as momentary (tap action, clear multi flags)
    pub fn resolve_as_momentary(&mut self) {
        self.is_multi = false;
        self.multikey = None;
    }

    /// Resolve as modifier (use multikey, clear multi flags)
    pub fn resolve_as_modifier(&mut self) {
        self.key = self.multikey;
        self.is_multi = false;
        self.multikey = None;
    }
}

impl fmt::Display for Keystate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Keystate(inkey={:?}, action={:?}, key={:?}, multikey={:?})",
            self.inkey, self.action, self.key, self.multikey
        )
    }
}

/// Runtime manager for multipurpose (tap/hold) keys
/// Handles the state machine for detecting tap vs hold behavior
#[derive(Debug)]
pub struct MultipurposeManager {
    /// All configured multipurpose modmaps (trigger_key -> modmap)
    modmaps: HashMap<Key, MultiModmap>,
    /// Currently active multipurpose state (if any)
    active: Option<ActiveMultipurpose>,
    /// Timeout duration for tap vs hold decision
    timeout: std::time::Duration,
}

/// Runtime state for active multipurpose key
#[derive(Debug, Clone)]
struct ActiveMultipurpose {
    /// The trigger key
    trigger_key: Key,
    /// The tap output key
    tap_output: Key,
    /// The hold output key  
    hold_output: Key,
    /// When the key was pressed
    press_time: std::time::Instant,
    /// Current sub-state
    state: MultipurposeSubState,
}

/// Sub-states within multipurpose handling
#[derive(Debug, Clone, Copy, PartialEq)]
enum MultipurposeSubState {
    /// Timing the press to determine tap vs hold
    Pending,
    /// Hold mode is active (timeout elapsed or interrupted)
    Hold,
}

/// Result of releasing a multipurpose key
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MultipurposeResult {
    /// Short press - output the tap key
    Tap(Key),
    /// Release the hold key (was being held)
    HoldRelease(Key),
}

impl MultipurposeManager {
    /// Create a new multipurpose manager with default 200ms timeout
    pub fn new() -> Self {
        Self {
            modmaps: HashMap::new(),
            active: None,
            timeout: std::time::Duration::from_millis(200),
        }
    }

    /// Create with custom timeout in milliseconds
    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            modmaps: HashMap::new(),
            active: None,
            timeout: std::time::Duration::from_millis(timeout_ms),
        }
    }

    /// Add a multipurpose modmap
    pub fn add_modmap(&mut self, modmap: MultiModmap) {
        // Store all mappings from this modmap
        for (trigger, (tap, hold)) in modmap.mappings.iter() {
            let mut mappings = HashMap::new();
            mappings.insert(*trigger, (*tap, *hold));
            
            // Preserve the conditional from the original modmap
            let single_modmap = if let Some(cond) = &modmap.conditional {
                MultiModmap::with_conditional(&modmap.name, mappings, cond.clone())
            } else {
                MultiModmap::new(&modmap.name, mappings)
            };
            
            self.modmaps.insert(*trigger, single_modmap);
        }
    }

    /// Set the timeout duration in milliseconds
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout = std::time::Duration::from_millis(timeout_ms);
    }

    /// Get the current timeout
    pub fn timeout(&self) -> std::time::Duration {
        self.timeout
    }

    /// Check if a key is a multipurpose trigger
    pub fn is_trigger(&self, key: Key) -> bool {
        self.modmaps.contains_key(&key)
    }

    /// Get the conditional string for a trigger key (if any)
    pub fn get_conditional(&self, key: Key) -> Option<&str> {
        self.modmaps.get(&key).and_then(|m| m.conditional())
    }

    /// Check if there's an active multipurpose key
    pub fn has_active(&self) -> bool {
        self.active.is_some()
    }

    /// Start a multipurpose sequence
    /// Returns true if this key started a multipurpose sequence
    pub fn start(&mut self, key: Key) -> bool {
        if let Some(modmap) = self.modmaps.get(&key) {
            // Get the tap/hold pair for this trigger key
            if let Some((tap_output, hold_output)) = modmap.get(key) {
                self.active = Some(ActiveMultipurpose {
                    trigger_key: key,
                    tap_output,
                    hold_output,
                    press_time: std::time::Instant::now(),
                    state: MultipurposeSubState::Pending,
                });
                return true;
            }
        }
        false
    }

    /// Check if the pending timeout has elapsed
    /// Returns Some(hold_key) if we should transition to hold mode
    pub fn check_timeout(&mut self) -> Option<Key> {
        if let Some(ref mut active) = self.active {
            if active.state == MultipurposeSubState::Pending {
                if active.press_time.elapsed() >= self.timeout {
                    // Transition to hold
                    active.state = MultipurposeSubState::Hold;
                    return Some(active.hold_output);
                }
            }
        }
        None
    }

    /// Handle another key being pressed while in pending state
    /// This causes an immediate transition to hold mode
    /// Returns Some((hold_key_press, new_key_to_process))
    pub fn interrupt_with_key(&mut self, new_key: Key) -> Option<(Key, Key)> {
        if let Some(ref mut active) = self.active {
            if active.state == MultipurposeSubState::Pending {
                // Transition to hold and output hold key
                let hold_output = active.hold_output;
                active.state = MultipurposeSubState::Hold;
                return Some((hold_output, new_key));
            }
        }
        None
    }

    /// Handle release of the multipurpose key
    /// Returns Some(result) - either tap or hold release
    pub fn release(&mut self) -> Option<MultipurposeResult> {
        if let Some(active) = self.active.take() {
            match active.state {
                MultipurposeSubState::Pending => {
                    // Short press = tap
                    let elapsed = active.press_time.elapsed();
                    if elapsed < self.timeout {
                        Some(MultipurposeResult::Tap(active.tap_output))
                    } else {
                        // Just at the boundary - treat as hold
                        Some(MultipurposeResult::HoldRelease(active.hold_output))
                    }
                }
                MultipurposeSubState::Hold => {
                    // Release the hold key
                    Some(MultipurposeResult::HoldRelease(active.hold_output))
                }
            }
        } else {
            None
        }
    }

    /// Get the hold key for the active modmap (for repeat handling)
    pub fn get_hold_key(&self) -> Option<Key> {
        self.active.as_ref().map(|a| a.hold_output)
    }

    /// Get the tap key for the active modmap
    pub fn get_tap_key(&self) -> Option<Key> {
        self.active.as_ref().map(|a| a.tap_output)
    }

    /// Get the trigger key for the active modmap
    pub fn get_trigger_key(&self) -> Option<Key> {
        self.active.as_ref().map(|a| a.trigger_key)
    }

    /// Check if currently in hold state
    pub fn is_hold_state(&self) -> bool {
        self.active
            .as_ref()
            .map(|a| a.state == MultipurposeSubState::Hold)
            .unwrap_or(false)
    }

    /// Check if currently in pending state
    pub fn is_pending_state(&self) -> bool {
        self.active
            .as_ref()
            .map(|a| a.state == MultipurposeSubState::Pending)
            .unwrap_or(false)
    }

    /// Clear any active state (e.g., on suspend)
    pub fn clear(&mut self) {
        self.active = None;
    }
}

impl Default for MultipurposeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Modifier;

    #[test]
    fn test_modmap() {
        let mut mappings = HashMap::new();
        mappings.insert(Key::from(30), Key::from(31)); // A -> S
        let modmap = Modmap::new("test", mappings);

        assert!(modmap.contains(Key::from(30)));
        assert_eq!(modmap.get(Key::from(30)), Some(Key::from(31)));
        assert!(!modmap.contains(Key::from(31)));
    }

    #[test]
    fn test_multi_modmap() {
        let mut mappings = HashMap::new();
        mappings.insert(Key::from(30), (Key::from(30), Key::from(29))); // A -> (A tap, Ctrl hold)
        let modmap = MultiModmap::new("test", mappings);

        assert!(modmap.contains(Key::from(30)));
        assert_eq!(
            modmap.get(Key::from(30)),
            Some((Key::from(30), Key::from(29)))
        );
    }

    #[test]
    fn test_keymap() {
        let mut keymap = Keymap::new("test");
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::from_single(ctrl, Key::from(30)); // Ctrl-A
        keymap.insert(combo.clone(), Key::from(31).into()); // -> S

        assert!(keymap.contains(&combo));
        assert_eq!(keymap.get(&combo), Some(&KeymapValue::Key(Key::from(31))));
    }

    #[test]
    fn test_keystate() {
        let keystate = Keystate::new(Key::from(30), Action::Press);
        assert!(keystate.key_is_pressed());
        assert!(!keystate.is_multi);

        let mut keystate = Keystate::new(Key::from(30), Action::Press);
        keystate.multikey = Some(Key::from(29));
        keystate.is_multi = true;
        keystate.resolve_as_momentary();
        assert!(!keystate.is_multi);
        assert!(keystate.multikey.is_none());
    }

    #[test]
    fn test_keymap_value() {
        let ctrl = Modifier::from_alias("Ctrl").unwrap();
        let combo = Combo::from_single(ctrl, Key::from(30));

        let value1: KeymapValue = combo.clone().into();
        let value2: KeymapValue = ComboHint::Bind.into();
        let value3: KeymapValue = Key::from(31).into();

        assert!(matches!(value1, KeymapValue::Combo(_)));
        assert!(matches!(value2, KeymapValue::ComboHint(_)));
        assert!(matches!(value3, KeymapValue::Key(_)));
    }

    // MultipurposeManager tests
    fn create_caps2esc_modmap() -> MultiModmap {
        let mut mappings = HashMap::new();
        // CAPSLOCK (58) -> (ESCAPE (1), RIGHT_CTRL (97))
        mappings.insert(Key::from(58), (Key::from(1), Key::from(97)));
        MultiModmap::new("Caps2Esc", mappings)
    }

    #[test]
    fn test_multipurpose_manager_creation() {
        let manager = MultipurposeManager::new();
        assert!(!manager.has_active());
        assert_eq!(manager.timeout().as_millis(), 200);
    }

    #[test]
    fn test_multipurpose_manager_with_timeout() {
        let manager = MultipurposeManager::with_timeout(500);
        assert_eq!(manager.timeout().as_millis(), 500);
    }

    #[test]
    fn test_multipurpose_trigger_detection() {
        let mut manager = MultipurposeManager::new();
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        assert!(manager.is_trigger(Key::from(58))); // CAPSLOCK
        assert!(!manager.is_trigger(Key::from(30))); // A
    }

    #[test]
    fn test_tap_detection() {
        let mut manager = MultipurposeManager::with_timeout(500); // 500ms timeout
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // Start the multipurpose sequence
        assert!(manager.start(Key::from(58)));
        assert!(manager.has_active());
        assert!(manager.is_pending_state());

        // Immediately release (before timeout)
        let result = manager.release();
        assert!(matches!(result, Some(MultipurposeResult::Tap(key)) if key == Key::from(1)));
        assert!(!manager.has_active());
    }

    #[test]
    fn test_hold_detection_via_timeout() {
        let mut manager = MultipurposeManager::with_timeout(10); // 10ms timeout
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // Start the sequence
        assert!(manager.start(Key::from(58)));

        // Wait for timeout
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Check timeout triggers hold
        let hold_key = manager.check_timeout();
        assert_eq!(hold_key, Some(Key::from(97)));
        assert!(manager.is_hold_state());

        // Release should output hold release
        let result = manager.release();
        assert!(matches!(result, Some(MultipurposeResult::HoldRelease(key)) if key == Key::from(97)));
    }

    #[test]
    fn test_interrupt_handling() {
        let mut manager = MultipurposeManager::with_timeout(500); // 500ms timeout
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // Start the sequence
        manager.start(Key::from(58));
        assert!(manager.is_pending_state());

        // Another key is pressed (interrupt)
        let interrupt = manager.interrupt_with_key(Key::from(30));
        assert!(interrupt.is_some());
        
        let (hold_key, new_key) = interrupt.unwrap();
        assert_eq!(hold_key, Key::from(97)); // Hold key press
        assert_eq!(new_key, Key::from(30));  // The interrupting key

        // Should now be in hold state
        assert!(manager.is_hold_state());
    }

    #[test]
    fn test_no_trigger_for_non_multipurpose_key() {
        let mut manager = MultipurposeManager::new();
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // Try to start with a non-multipurpose key
        assert!(!manager.start(Key::from(30)));
        assert!(!manager.has_active());
    }

    #[test]
    fn test_clear_active() {
        let mut manager = MultipurposeManager::new();
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        manager.start(Key::from(58));
        assert!(manager.has_active());

        manager.clear();
        assert!(!manager.has_active());
    }

    #[test]
    fn test_get_keys() {
        let mut manager = MultipurposeManager::new();
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // No active modmap
        assert_eq!(manager.get_hold_key(), None);
        assert_eq!(manager.get_tap_key(), None);

        // Start and get keys
        manager.start(Key::from(58));
        assert_eq!(manager.get_hold_key(), Some(Key::from(97)));
        assert_eq!(manager.get_tap_key(), Some(Key::from(1)));
    }

    #[test]
    fn test_hold_release_after_interrupt() {
        let mut manager = MultipurposeManager::with_timeout(500);
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // Start and interrupt
        assert!(manager.start(Key::from(58)));
        let result = manager.interrupt_with_key(Key::from(30));
        assert!(result.is_some());
        let (hold_key, new_key) = result.unwrap();
        assert_eq!(hold_key, Key::from(97)); // RIGHT_CTRL
        assert_eq!(new_key, Key::from(30)); // The interrupting key

        // Should be in hold state
        assert!(manager.is_hold_state());

        // Release the trigger key - should emit hold release
        let result = manager.release();
        assert_eq!(result, Some(MultipurposeResult::HoldRelease(Key::from(97))));

        // No longer active
        assert!(!manager.has_active());
    }

    #[test]
    fn test_multiple_multipurpose_modmaps() {
        let mut manager = MultipurposeManager::new();

        // Add Caps2Esc
        let caps_modmap = create_caps2esc_modmap();
        manager.add_modmap(caps_modmap);

        // Add Enter2Ctrl
        let mut enter_mappings = HashMap::new();
        enter_mappings.insert(Key::from(28), (Key::from(28), Key::from(97))); // Enter -> (Enter, RCtrl)
        let enter_modmap = MultiModmap::new("Enter2Ctrl", enter_mappings);
        manager.add_modmap(enter_modmap);

        // Both should be triggers
        assert!(manager.is_trigger(Key::from(58))); // CAPSLOCK
        assert!(manager.is_trigger(Key::from(28))); // ENTER
        assert!(!manager.is_trigger(Key::from(30))); // A - not a trigger

        // Start with Caps
        assert!(manager.start(Key::from(58)));
        assert_eq!(manager.get_tap_key(), Some(Key::from(1))); // ESCAPE

        // Release and start with Enter
        manager.release();
        assert!(manager.start(Key::from(28)));
        assert_eq!(manager.get_tap_key(), Some(Key::from(28))); // ENTER
        assert_eq!(manager.get_hold_key(), Some(Key::from(97))); // RCTRL
    }

    #[test]
    fn test_release_without_active() {
        let mut manager = MultipurposeManager::new();
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // Release without any active modmap should return None
        assert_eq!(manager.release(), None);
    }

    #[test]
    fn test_interrupt_without_active() {
        let mut manager = MultipurposeManager::new();
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // Interrupt without any active modmap should return None
        assert_eq!(manager.interrupt_with_key(Key::from(30)), None);
    }

    #[test]
    fn test_start_non_trigger_key() {
        let mut manager = MultipurposeManager::new();
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // Starting a non-trigger key should return false
        assert!(!manager.start(Key::from(30))); // A key
        assert!(!manager.has_active());
    }

    #[test]
    fn test_get_trigger_key() {
        let mut manager = MultipurposeManager::new();
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // No active modmap
        assert_eq!(manager.get_trigger_key(), None);

        // Start and get trigger key
        manager.start(Key::from(58));
        assert_eq!(manager.get_trigger_key(), Some(Key::from(58)));
    }

    #[test]
    fn test_pending_state_check() {
        let mut manager = MultipurposeManager::with_timeout(500);
        let modmap = create_caps2esc_modmap();
        manager.add_modmap(modmap);

        // Not pending initially
        assert!(!manager.is_pending_state());

        // Start - should be pending
        manager.start(Key::from(58));
        assert!(manager.is_pending_state());

        // After interrupt - should be hold state, not pending
        manager.interrupt_with_key(Key::from(30));
        assert!(!manager.is_pending_state());
        assert!(manager.is_hold_state());
    }
}

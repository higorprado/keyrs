// Xwaykeyz Pure Rust uinput Output Layer
// Virtual device creation and key event emission

use super::cache::OutputCache;
use super::combo::calculate_combo_actions;
use super::state::PressedKeyState;
use crate::key::{ascii_to_key, key_from_name};
use crate::mapping::ActionStep;
use crate::{Action, Combo, ComboHint, Key, Modifier};
use std::sync::OnceLock;

#[cfg(feature = "pure-rust")]
use evdev::{EventType, InputEvent};

/// Virtual uinput device for key output
#[cfg(feature = "pure-rust")]
pub struct VirtualDevice {
    device: evdev::uinput::VirtualDevice,
    pressed_keys: PressedKeyState,
    pressed_modifiers: PressedKeyState,
    cache: OutputCache,
    key_pre_delay_ms: u64,
    key_post_delay_ms: u64,
}

/// Error types for uinput operations
#[derive(Debug, thiserror::Error)]
pub enum UInputError {
    #[error("Failed to create virtual device: {0}")]
    DeviceCreation(String),

    #[error("Failed to write event: {0}")]
    WriteError(String),

    #[error("Device not initialized")]
    NotInitialized,
}

#[cfg(feature = "pure-rust")]
impl VirtualDevice {
    fn debug_output_enabled() -> bool {
        static DEBUG_OUTPUT: OnceLock<bool> = OnceLock::new();
        *DEBUG_OUTPUT.get_or_init(|| {
            std::env::var("XWAYKEYZ_DEBUG_OUTPUT")
                .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
                .unwrap_or(false)
        })
    }

    fn debug_output_log(&self, message: &str) {
        if Self::debug_output_enabled() {
            eprintln!("[OUTPUT-DEBUG] {}", message);
        }
    }

    /// Create a new virtual uinput device
    pub fn new() -> Result<Self, UInputError> {
        use evdev::uinput::VirtualDeviceBuilder;
        use evdev::AttributeSet;

        // Build the virtual device with keyboard support
        let mut keys = AttributeSet::new();
        // Add all standard keyboard keys (0-255)
        for code in 0..256u16 {
            keys.insert(evdev::Key::new(code));
        }

        let device = VirtualDeviceBuilder::new()
            .map_err(|e: std::io::Error| UInputError::DeviceCreation(e.to_string()))?
            .name("Keyrs (virtual) Keyboard")
            .with_keys(&keys)
            .map_err(|e: std::io::Error| UInputError::DeviceCreation(e.to_string()))?
            .build()
            .map_err(|e: std::io::Error| UInputError::DeviceCreation(e.to_string()))?;

        Ok(Self {
            device,
            pressed_keys: PressedKeyState::new(),
            pressed_modifiers: PressedKeyState::new(),
            cache: OutputCache::new(),
            key_pre_delay_ms: 0,
            key_post_delay_ms: 0,
        })
    }

    /// Configure output throttle delays in milliseconds.
    pub fn set_throttle_delays(&mut self, key_pre_delay_ms: u64, key_post_delay_ms: u64) {
        self.key_pre_delay_ms = key_pre_delay_ms;
        self.key_post_delay_ms = key_post_delay_ms;
    }

    /// Write a single key event to the virtual device
    fn write_key_event(&mut self, key: Key, action: Action) -> Result<(), UInputError> {
        let value = match action {
            Action::Press => 1,
            Action::Release => 0,
            Action::Repeat => 2,
        };

        let key_code = key.code();
        let key_event = InputEvent::new(EventType::KEY, key_code as u16, value);
        // SYN event is required for the kernel to process the key event
        let syn_event = InputEvent::new(EventType::SYNCHRONIZATION, 0, 0);

        self.device
            .emit(&[key_event, syn_event])
            .map_err(|e: std::io::Error| UInputError::WriteError(e.to_string()))?;

        // Update pressed state
        if Modifier::is_key_modifier(key) {
            match action {
                Action::Press => self.pressed_modifiers.add(key),
                Action::Release => self.pressed_modifiers.remove(key),
                Action::Repeat => {}
            }
        } else {
            match action {
                Action::Press => self.pressed_keys.add(key),
                Action::Release => self.pressed_keys.remove(key),
                Action::Repeat => {}
            }
        }

        Ok(())
    }

    /// Send a key action with optional delays
    pub fn send_key_action(&mut self, key: Key, action: Action) -> Result<(), UInputError> {
        if Self::debug_output_enabled() {
            self.debug_output_log(&format!(
                "send_key_action key={:?} action={:?} pre={}ms post={}ms",
                key, action, self.key_pre_delay_ms, self.key_post_delay_ms
            ));
        }
        if self.key_pre_delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.key_pre_delay_ms));
        }
        self.write_key_event(key, action)?;
        if self.key_post_delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.key_post_delay_ms));
        }
        Ok(())
    }

    fn tap_key(&mut self, key: Key) -> Result<(), UInputError> {
        self.send_key_action(key, Action::Press)?;
        self.send_key_action(key, Action::Release)?;
        Ok(())
    }

    fn key_for_unicode_digit(ch: char) -> Result<Key, UInputError> {
        let key_name = match ch {
            '0'..='9' | 'a'..='f' => ch.to_string(),
            'A'..='F' => ch.to_ascii_lowercase().to_string(),
            _ => {
                return Err(UInputError::WriteError(format!(
                    "Unsupported Unicode hex digit: '{}'",
                    ch
                )))
            }
        };

        key_from_name(&key_name).ok_or_else(|| {
            UInputError::WriteError(format!("Unable to resolve key for Unicode digit '{}'", ch))
        })
    }

    fn key_required(name: &str) -> Result<Key, UInputError> {
        key_from_name(name)
            .ok_or_else(|| UInputError::WriteError(format!("Required key '{}' not found", name)))
    }

    fn ascii_key_and_shift(ch: char) -> Option<(Key, bool)> {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            return key_from_name(&ch.to_string()).map(|k| (k, false));
        }
        if ch.is_ascii_uppercase() {
            return key_from_name(&ch.to_ascii_lowercase().to_string()).map(|k| (k, true));
        }

        match ch {
            ' ' => Some((ascii_to_key(' ')?, false)),
            '\n' => Some((Self::key_required("ENTER").ok()?, false)),
            '\t' => Some((Self::key_required("TAB").ok()?, false)),
            '-' => Some((ascii_to_key('-')?, false)),
            '_' => Some((ascii_to_key('-')?, true)),
            '=' => Some((ascii_to_key('=')?, false)),
            '+' => Some((ascii_to_key('=')?, true)),
            '[' => Some((ascii_to_key('[')?, false)),
            '{' => Some((ascii_to_key('[')?, true)),
            ']' => Some((ascii_to_key(']')?, false)),
            '}' => Some((ascii_to_key(']')?, true)),
            '\\' => Some((ascii_to_key('\\')?, false)),
            '|' => Some((ascii_to_key('\\')?, true)),
            ';' => Some((ascii_to_key(';')?, false)),
            ':' => Some((ascii_to_key(';')?, true)),
            '\'' => Some((ascii_to_key('\'')?, false)),
            '"' => Some((ascii_to_key('\'')?, true)),
            ',' => Some((ascii_to_key(',')?, false)),
            '<' => Some((ascii_to_key(',')?, true)),
            '.' => Some((ascii_to_key('.')?, false)),
            '>' => Some((ascii_to_key('.')?, true)),
            '/' => Some((ascii_to_key('/')?, false)),
            '?' => Some((ascii_to_key('/')?, true)),
            '`' => Some((ascii_to_key('`')?, false)),
            '~' => Some((ascii_to_key('`')?, true)),
            '!' => Some((key_from_name("1")?, true)),
            '@' => Some((key_from_name("2")?, true)),
            '#' => Some((key_from_name("3")?, true)),
            '$' => Some((key_from_name("4")?, true)),
            '%' => Some((key_from_name("5")?, true)),
            '^' => Some((key_from_name("6")?, true)),
            '&' => Some((key_from_name("7")?, true)),
            '*' => Some((key_from_name("8")?, true)),
            '(' => Some((key_from_name("9")?, true)),
            ')' => Some((key_from_name("0")?, true)),
            _ => None,
        }
    }

    fn send_ascii_char(&mut self, ch: char) -> Result<bool, UInputError> {
        let Some((key, needs_shift)) = Self::ascii_key_and_shift(ch) else {
            return Ok(false);
        };

        if needs_shift {
            let left_shift = Self::key_required("LEFT_SHIFT")?;
            self.send_key_action(left_shift, Action::Press)?;
            self.tap_key(key)?;
            self.send_key_action(left_shift, Action::Release)?;
        } else {
            self.tap_key(key)?;
        }

        Ok(true)
    }

    /// Send a Unicode character via Linux's Ctrl+Shift+U compose sequence.
    pub fn send_unicode(&mut self, codepoint: u32) -> Result<(), UInputError> {
        if char::from_u32(codepoint).is_none() {
            return Err(UInputError::WriteError(format!(
                "Invalid Unicode codepoint: 0x{codepoint:X}"
            )));
        }

        let hex = format!("{codepoint:x}");
        let left_ctrl = Self::key_required("LEFT_CTRL")?;
        let left_shift = Self::key_required("LEFT_SHIFT")?;
        let u_key = Self::key_required("U")?;
        let enter = Self::key_required("ENTER")?;

        // Prevent currently held modifiers from interfering with Unicode composition.
        let held_modifiers = self.pressed_modifiers.get_all();
        for modifier in held_modifiers.iter().rev() {
            self.send_key_action(*modifier, Action::Release)?;
        }

        // Trigger compose mode: Ctrl+Shift+U
        self.send_key_action(left_ctrl, Action::Press)?;
        self.send_key_action(left_shift, Action::Press)?;
        self.tap_key(u_key)?;
        self.send_key_action(left_shift, Action::Release)?;
        self.send_key_action(left_ctrl, Action::Release)?;

        // Type hexadecimal codepoint and commit with Enter.
        for ch in hex.chars() {
            let digit_key = Self::key_for_unicode_digit(ch)?;
            self.tap_key(digit_key)?;
        }
        self.tap_key(enter)?;

        // Restore modifiers that were held before Unicode entry.
        for modifier in &held_modifiers {
            self.send_key_action(*modifier, Action::Press)?;
        }

        Ok(())
    }

    /// Send text using direct ASCII key events when possible, with Unicode compose fallback.
    pub fn send_text(&mut self, text: &str) -> Result<(), UInputError> {
        self.debug_output_log(&format!("send_text start len={} text='{}'", text.len(), text));
        // Prevent currently held modifiers from interfering with text emission.
        let held_modifiers = self.pressed_modifiers.get_all();
        if Self::debug_output_enabled() {
            self.debug_output_log(&format!(
                "send_text releasing held modifiers: {:?}",
                held_modifiers
            ));
        }
        for modifier in held_modifiers.iter().rev() {
            self.send_key_action(*modifier, Action::Release)?;
        }

        for (idx, ch) in text.chars().enumerate() {
            if !self.send_ascii_char(ch)? {
                self.debug_output_log(&format!(
                    "send_text char[{}]='{}' path=unicode",
                    idx, ch
                ));
                self.send_unicode(ch as u32)?;
            } else if Self::debug_output_enabled() {
                self.debug_output_log(&format!("send_text char[{}]='{}' path=ascii", idx, ch));
            }

            // Some apps/shells drop characters when virtual key events arrive
            // with zero gap. Add a minimal pacing fallback unless a post delay
            // is already configured.
            if self.key_post_delay_ms == 0 {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }

        // Restore previously held modifiers.
        for modifier in &held_modifiers {
            self.send_key_action(*modifier, Action::Press)?;
        }
        self.debug_output_log("send_text end");
        Ok(())
    }

    /// Send a combo sequence
    pub fn send_combo(&mut self, combo: &Combo) -> Result<(), UInputError> {
        let modifiers = combo.modifiers().to_vec();
        let main_key = combo.key();

        // Get currently pressed modifier keys
        let pressed_mods = self.pressed_modifiers.get_all();

        // Calculate the action sequence
        let actions = calculate_combo_actions(&modifiers, main_key, &pressed_mods);

        // Release modifiers that need to be lifted
        for key in &actions.modifiers_to_release {
            self.send_key_action(*key, Action::Release)?;
        }

        // Press modifiers that need to be pressed
        for key in &actions.modifiers_to_press {
            self.send_key_action(*key, Action::Press)?;
        }

        // Press and release the main key
        self.send_key_action(main_key, Action::Press)?;
        self.send_key_action(main_key, Action::Release)?;

        // Release the pressed modifiers
        for key in actions.modifiers_to_press.iter().rev() {
            self.send_key_action(*key, Action::Release)?;
        }

        // Restore modifiers that were released
        for key in &actions.modifiers_to_restore {
            self.send_key_action(*key, Action::Press)?;
        }

        Ok(())
    }

    /// Send a combo while preserving currently-held modifiers.
    /// This is used by `bind` sequence semantics where we avoid lifting
    /// physically held modifiers before dispatching the combo.
    pub fn send_combo_bound(&mut self, combo: &Combo) -> Result<(), UInputError> {
        let modifiers = combo.modifiers().to_vec();
        let main_key = combo.key();
        let pressed_mods = self.pressed_modifiers.get_all();

        // Press only missing target modifiers; keep existing held modifiers as-is.
        let mut newly_pressed = Vec::new();
        for modifier in modifiers {
            let already_pressed = modifier.keys().iter().any(|k| pressed_mods.contains(k));
            if !already_pressed {
                let key_to_press = modifier.key();
                self.send_key_action(key_to_press, Action::Press)?;
                newly_pressed.push(key_to_press);
            }
        }

        self.send_key_action(main_key, Action::Press)?;
        self.send_key_action(main_key, Action::Release)?;

        // Release only modifiers we introduced for this bound combo.
        for key in newly_pressed.into_iter().rev() {
            self.send_key_action(key, Action::Release)?;
        }

        Ok(())
    }

    fn execute_sequence_step(&mut self, step: &ActionStep, bind_next: &mut bool) -> Result<(), UInputError> {
        if Self::debug_output_enabled() {
            self.debug_output_log(&format!("execute_sequence_step {:?}", step));
        }
        match step {
            ActionStep::Combo(combo) => {
                if *bind_next {
                    *bind_next = false;
                    self.send_combo_bound(combo)
                } else {
                    self.send_combo(combo)
                }
            }
            ActionStep::Text(text) => self.send_text(text),
            ActionStep::DelayMs(ms) => {
                std::thread::sleep(std::time::Duration::from_millis(*ms));
                Ok(())
            }
            ActionStep::Ignore => Ok(()),
            ActionStep::Bind => {
                *bind_next = true;
                Ok(())
            }
            ActionStep::SetSetting { .. } => Ok(()),
        }
    }

    fn execute_sequence(&mut self, steps: &[ActionStep]) -> Result<(), UInputError> {
        self.debug_output_log(&format!("execute_sequence start steps={}", steps.len()));
        let has_bind = steps.iter().any(|step| matches!(step, ActionStep::Bind));
        // For non-bind sequences, release held modifiers for the whole sequence
        // to avoid compositor/app shortcuts consuming macro steps.
        // For bind sequences, keep held modifiers because bind semantics depend on them.
        let held_modifiers = if has_bind {
            Vec::new()
        } else {
            self.pressed_modifiers.get_all()
        };
        if Self::debug_output_enabled() {
            self.debug_output_log(&format!(
                "execute_sequence has_bind={} held_modifiers_before={:?}",
                has_bind, held_modifiers
            ));
        }
        for modifier in held_modifiers.iter().rev() {
            self.send_key_action(*modifier, Action::Release)?;
        }

        let sequence_result = (|| -> Result<(), UInputError> {
            let mut bind_next = false;
            for step in steps {
                self.execute_sequence_step(step, &mut bind_next)?;
            }
            Ok(())
        })();

        let mut restore_error: Option<UInputError> = None;
        for modifier in &held_modifiers {
            if let Err(e) = self.send_key_action(*modifier, Action::Press) {
                restore_error = Some(e);
                break;
            }
        }

        match (sequence_result, restore_error) {
            (Err(e), _) => Err(e),
            (Ok(_), Some(e)) => Err(e),
            (Ok(_), None) => {
                self.debug_output_log("execute_sequence end");
                Ok(())
            }
        }
    }

    /// Process a transform result and send appropriate output
    /// 
    /// # Arguments
    /// * `result` - The transform result to process
    /// * `action` - The original input action (Press/Release/Repeat)
    pub fn process_transform_result(
        &mut self,
        result: &TransformResultOutput,
        action: Action,
    ) -> Result<(), UInputError> {
        match result {
            TransformResultOutput::Passthrough(key) => {
                // Check if this is a regular key (not a modifier) and modifiers are held
                let is_modifier = Modifier::is_key_modifier(*key);
                let held_modifier_keys = self.pressed_modifiers.get_all();
                
                if !is_modifier && action == Action::Press && !held_modifier_keys.is_empty() {
                    // Regular key pressed while modifiers are held
                    self.send_key_action(*key, Action::Press)?;
                    self.pressed_keys.add(*key);
                } else if !is_modifier && action == Action::Release && self.pressed_keys.is_pressed(*key) {
                    // Key was pressed while modifier was held, now release it
                    self.send_key_action(*key, Action::Release)?;
                    self.pressed_keys.remove(*key);
                } else {
                    // Send key with the original action
                    self.send_key_action(*key, action)?;
                }
            }
            TransformResultOutput::Remapped(key) => {
                // Check if this is a regular key (not a modifier) and modifiers are held
                let is_modifier = Modifier::is_key_modifier(*key);
                let held_modifier_keys = self.pressed_modifiers.get_all();
                let was_pressed = if is_modifier {
                    self.pressed_modifiers.is_pressed(*key)
                } else {
                    self.pressed_keys.is_pressed(*key)
                };

                if action == Action::Release && !was_pressed {
                    // Release without a tracked press (common for tap-on-release remaps).
                    // Emit a synthetic tap for both regular keys and modifiers.
                    self.send_key_action(*key, Action::Press)?;
                    self.send_key_action(*key, Action::Release)?;
                } else if !is_modifier && action == Action::Press && !held_modifier_keys.is_empty() {
                    // Regular key pressed while modifiers are held
                    self.send_key_action(*key, Action::Press)?;
                    self.pressed_keys.add(*key);
                } else if !is_modifier && action == Action::Release && self.pressed_keys.is_pressed(*key) {
                    // Key was pressed while modifier was held, now release it
                    self.send_key_action(*key, Action::Release)?;
                    self.pressed_keys.remove(*key);
                } else {
                    // Send key with the original action
                    self.send_key_action(*key, action)?;
                }
            }
            TransformResultOutput::ComboKey(key) => {
                // ComboKey behaves like a hotkey output: one synthetic tap on press.
                // Ignore release/repeat to avoid stuck keys and repeated characters.
                if action == Action::Press {
                    self.tap_key(*key)?;
                }
            }
            TransformResultOutput::Combo(combo) => {
                // Send the full combo
                self.send_combo(combo)?;
            }
            TransformResultOutput::Sequence(steps) => {
                if action == Action::Press {
                    if Self::debug_output_enabled() {
                        self.debug_output_log(&format!(
                            "process_transform_result Sequence press with {} steps",
                            steps.len()
                        ));
                    }
                    self.execute_sequence(steps)?;
                }
            }
            TransformResultOutput::Hint(_hint) => {
                // Hints are special - they don't produce immediate output
                // They're used for state tracking (Bind, EscapeNext, etc.)
            }
            TransformResultOutput::Unicode(codepoint) => {
                if action == Action::Press {
                    self.send_unicode(*codepoint)?;
                }
            }
            TransformResultOutput::Text(text) => {
                if action == Action::Press {
                    self.send_text(text)?;
                }
            }
            TransformResultOutput::Suppress => {
                // Don't send anything
            }
            TransformResultOutput::Suspend => {
                // Suspend mode - release all keys
                self.release_all()?;
            }
        }

        Ok(())
    }

    /// Release all pressed keys (for shutdown/suspend)
    pub fn release_all(&mut self) -> Result<(), UInputError> {
        // Release regular keys first (in reverse order - LIFO)
        let all_keys = self.pressed_keys.get_all();
        for key in all_keys.into_iter().rev() {
            self.send_key_action(key, Action::Release)?;
        }

        // Release modifiers (in reverse order)
        let all_mods = self.pressed_modifiers.get_all();
        for key in all_mods.into_iter().rev() {
            self.send_key_action(key, Action::Release)?;
        }

        Ok(())
    }

    /// Check if a modifier is pressed
    pub fn is_mod_pressed(&self, key: Key) -> bool {
        self.pressed_modifiers.is_pressed(key)
    }

    /// Check if any key is pressed
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.is_pressed(key)
    }

    /// Get the number of pressed keys
    pub fn pressed_key_count(&self) -> usize {
        self.pressed_keys.len()
    }

    /// Get the number of pressed modifiers
    pub fn pressed_modifier_count(&self) -> usize {
        self.pressed_modifiers.len()
    }

    /// Clear the output cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Close the virtual device
    pub fn close(self) -> Result<(), UInputError> {
        // Release all keys before closing
        let mut device = self;
        device.release_all()?;
        drop(device);
        Ok(())
    }
}

/// Simplified version of TransformResult for output processing
#[derive(Debug, Clone, PartialEq)]
pub enum TransformResultOutput {
    /// Passthrough - send the key as-is
    Passthrough(Key),
    /// Remapped to a different key
    Remapped(Key),
    /// Combo matched with a key output
    ComboKey(Key),
    /// Combo matched with a combo output (multi-key)
    Combo(Combo),
    /// Combo matched with a multi-step sequence output
    Sequence(Vec<ActionStep>),
    /// Special hint (Bind, EscapeNext, etc.)
    Hint(ComboHint),
    /// Suppressed - don't send anything
    Suppress,
    /// Suspend mode activated
    Suspend,
    /// Unicode character output (for international characters)
    Unicode(u32),
    /// Text output (typed as Unicode sequence)
    Text(String),
}

impl TransformResultOutput {
    /// Create from the transform engine's TransformResult
    pub fn from_transform_result(result: &crate::transform::TransformResult) -> Self {
        match result {
            crate::transform::TransformResult::Passthrough(key) => Self::Passthrough(*key),
            crate::transform::TransformResult::Remapped(key) => Self::Remapped(*key),
            crate::transform::TransformResult::ComboKey(key) => Self::ComboKey(*key),
            crate::transform::TransformResult::Combo(combo) => Self::Combo(combo.clone()),
            crate::transform::TransformResult::Sequence(steps) => Self::Sequence(steps.clone()),
            crate::transform::TransformResult::Hint(hint) => Self::Hint(*hint),
            crate::transform::TransformResult::Suppress => Self::Suppress,
            crate::transform::TransformResult::Suspend => Self::Suspend,
            crate::transform::TransformResult::Unicode(codepoint) => Self::Unicode(*codepoint),
            crate::transform::TransformResult::Text(text) => Self::Text(text.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_virtual_device_creation() {
        // This test requires actual uinput access
        // It may fail in containerized environments
        match VirtualDevice::new() {
            Ok(_device) => {
                // Successfully created
                assert!(true);
            }
            Err(_) => {
                // May fail in CI/container environments
                // That's ok for now
            }
        }
    }

    #[test]
    fn test_modifier_key_tracking() {
        // Test that verifies modifier keys are tracked correctly
        // Simulates: CAPSLOCK remapped to LEFT_CTRL, then C pressed
        
        let mut state = PressedKeyState::new();
        let ctrl_key = Key::from(29); // LEFT_CTRL
        let c_key = Key::from(46); // C
        
        // Step 1: CAPSLOCK press (remapped to LEFT_CTRL)
        // This would come through as Remapped(Key(29))
        assert!(Modifier::is_key_modifier(ctrl_key), "Key(29) should be a modifier");
        state.add(ctrl_key);
        assert!(state.is_pressed(ctrl_key));
        
        // Step 2: C press 
        // This comes through as Passthrough(Key(46))
        assert!(!Modifier::is_key_modifier(c_key), "Key(46) should NOT be a modifier");
        
        // At this point, the modifier should be tracked
        let held_modifiers = state.get_all();
        assert_eq!(held_modifiers.len(), 1);
        assert!(held_modifiers.contains(&ctrl_key));
        
        // Step 3: Verify modifier can be converted back
        let mods: Vec<Modifier> = held_modifiers
            .iter()
            .filter_map(|k| Modifier::from_key(*k))
            .collect();
        assert!(!mods.is_empty(), "Should be able to convert Key(29) to Modifier");
    }

    #[test]
    fn test_key_29_is_modifier() {
        // Specific test for Key(29) = LEFT_CTRL
        let key = Key::from(29);
        assert!(Modifier::is_key_modifier(key), "Key(29) LEFT_CTRL must be detected as modifier");
    }

    #[test]
    fn test_transform_result_output_creation() {
        let key = Key::from(30); // A
        let output = TransformResultOutput::Passthrough(key);
        assert_eq!(output, TransformResultOutput::Passthrough(key));
    }

    #[test]
    fn test_transform_result_output_combo() {
        let combo = Combo::new(None, Key::from(30));
        let output = TransformResultOutput::Combo(combo);
        match output {
            TransformResultOutput::Combo(c) => {
                assert_eq!(c.key(), Key::from(30));
            }
            _ => panic!("Expected Combo variant"),
        }
    }

    #[test]
    fn test_transform_result_output_unicode() {
        // Test Unicode output with codepoint for é (0x00E9)
        let output = TransformResultOutput::Unicode(0x00E9);
        assert_eq!(output, TransformResultOutput::Unicode(0x00E9));

        // Test with common Unicode characters
        let tests = [
            (0x00E0, "à"),   // grave + a
            (0x00E9, "é"),   // acute + e
            (0x00F1, "ñ"),   // tilde + n
            (0x00FC, "ü"),   // umlaut + u
            (0x0161, "š"),   // caron + s
        ];

        for (codepoint, _name) in tests {
            let output = TransformResultOutput::Unicode(codepoint);
            assert_eq!(output, TransformResultOutput::Unicode(codepoint));
        }
    }

    #[test]
    fn test_transform_result_output_text() {
        let output = TransformResultOutput::Text("hello".to_string());
        assert_eq!(output, TransformResultOutput::Text("hello".to_string()));
    }

    #[test]
    fn test_transform_result_output_from_transform() {
        use crate::transform::TransformResult;

        let key = Key::from(30);
        let tr = TransformResult::Passthrough(key);
        let output = TransformResultOutput::from_transform_result(&tr);

        assert_eq!(output, TransformResultOutput::Passthrough(key));

        let tr_text = TransformResult::Text("abc".to_string());
        let output_text = TransformResultOutput::from_transform_result(&tr_text);
        assert_eq!(output_text, TransformResultOutput::Text("abc".to_string()));

        let tr_sequence = TransformResult::Sequence(vec![
            ActionStep::DelayMs(50),
            ActionStep::Bind,
            ActionStep::Text("x".to_string()),
        ]);
        let output_sequence = TransformResultOutput::from_transform_result(&tr_sequence);
        assert_eq!(
            output_sequence,
            TransformResultOutput::Sequence(vec![
                ActionStep::DelayMs(50),
                ActionStep::Bind,
                ActionStep::Text("x".to_string())
            ])
        );
    }

    #[test]
    fn test_ascii_key_and_shift_mapping() {
        assert_eq!(VirtualDevice::ascii_key_and_shift('a'), Some((Key::from(30), false)));
        assert_eq!(VirtualDevice::ascii_key_and_shift('A'), Some((Key::from(30), true)));
        assert_eq!(VirtualDevice::ascii_key_and_shift('-'), Some((Key::from(12), false)));
        assert_eq!(VirtualDevice::ascii_key_and_shift('_'), Some((Key::from(12), true)));
        assert_eq!(VirtualDevice::ascii_key_and_shift('!'), Some((Key::from(2), true)));
    }
}

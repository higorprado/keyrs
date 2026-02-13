// Xwaykeyz Pure Rust Transform Engine - Complete Implementation
// Complete event transformation pipeline without Python dependencies
//
// This module implements pure Rust transform engine that eliminates
// dual state storage problem present in Python runtime.
//
// Features:
// - Complete modmap lookup with conditional evaluation
// - Multi-modmap (tap/hold) logic
// - Full combo matching with modifier expansion
// - Nested keymap support
// - Special hints handling (Bind, EscapeNext, Ignore, SetMark)
// - Optimized repeat cache
// - Window context integration

#[cfg(feature = "pure-rust")]
use std::sync::Arc;

#[cfg(feature = "pure-rust")]
use std::time::{Duration, Instant};
#[cfg(feature = "pure-rust")]
use std::collections::HashSet;

#[cfg(feature = "pure-rust")]
use parking_lot::RwLock;

use crate::mapping::{ActionStep, Keymap, KeymapValue, Modmap, MultiModmap, MultipurposeManager, MultipurposeResult};
use crate::transform::deadkeys::DeadKeyState;
use crate::transform::ComboMatchResult;
use crate::window::WindowContextProvider;
use crate::{Action, Combo, ComboHint, Key, Keystore, Modifier};

/// Configuration for transform engine
#[derive(Debug, Clone)]
pub struct TransformConfig {
    /// Modifier maps (first is default, rest are conditional)
    pub modmaps: Vec<Modmap>,
    /// Multi-modifier maps
    pub multimodmaps: Vec<MultiModmap>,
    /// Keymaps for combo matching
    pub keymaps: Vec<Keymap>,
    /// Suspend key (optional)
    pub suspend_key: Option<Key>,
    /// Multipurpose timeout (milliseconds)
    pub multipurpose_timeout: Option<u64>,
    /// Suspend timeout (milliseconds)
    pub suspend_timeout: Option<u64>,
}

impl Default for TransformConfig {
    fn default() -> Self {
        use std::collections::HashMap;
        Self {
            modmaps: vec![Modmap::new("default", HashMap::new())],
            multimodmaps: vec![],
            keymaps: vec![],
            suspend_key: None,
            multipurpose_timeout: Some(500),
            suspend_timeout: Some(1000),
        }
    }
}

/// Result of transforming a single key event
#[derive(Debug, Clone, PartialEq)]
pub enum TransformResult {
    /// Passthrough - send key as-is
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
    /// Unicode character output (for international/dead key support)
    Unicode(u32),
    /// Text output (typed as Unicode characters in sequence)
    Text(String),
}

/// Window context for conditional modmap/keymap evaluation
#[derive(Debug, Clone, Default)]
pub struct WindowContext {
    /// Window class (WM_CLASS on X11)
    pub wm_class: Option<String>,
    /// Window name (WM_NAME on X11)
    pub wm_name: Option<String>,
    /// Active device name for current event source
    pub device_name: Option<String>,
    /// Num Lock state
    pub numlock_on: bool,
    /// Caps Lock state
    pub capslock_on: bool,
    /// Keyboard type for keyboard-specific modmaps
    pub keyboard_type: Option<crate::input::KeyboardType>,
    /// Settings for feature toggles
    pub settings: crate::settings::Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConditionToken {
    LParen,
    RParen,
    And,
    Or,
    Not,
    Eq,
    Match,
    Ident(String),
    StringLit(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConditionOp {
    Eq,
    Match,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConditionExpr {
    And(Box<ConditionExpr>, Box<ConditionExpr>),
    Or(Box<ConditionExpr>, Box<ConditionExpr>),
    Not(Box<ConditionExpr>),
    Predicate {
        field: String,
        op: Option<ConditionOp>,
        value: Option<String>,
    },
}

struct ConditionParser {
    tokens: Vec<ConditionToken>,
    pos: usize,
}

impl ConditionParser {
    fn new(tokens: Vec<ConditionToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse(mut self) -> Option<ConditionExpr> {
        let expr = self.parse_or()?;
        if self.peek().is_some() {
            return None;
        }
        Some(expr)
    }

    fn parse_or(&mut self) -> Option<ConditionExpr> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(ConditionToken::Or)) {
            self.next();
            let right = self.parse_and()?;
            left = ConditionExpr::Or(Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn parse_and(&mut self) -> Option<ConditionExpr> {
        let mut left = self.parse_not()?;
        while matches!(self.peek(), Some(ConditionToken::And)) {
            self.next();
            let right = self.parse_not()?;
            left = ConditionExpr::And(Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn parse_not(&mut self) -> Option<ConditionExpr> {
        if matches!(self.peek(), Some(ConditionToken::Not)) {
            self.next();
            let inner = self.parse_not()?;
            return Some(ConditionExpr::Not(Box::new(inner)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Option<ConditionExpr> {
        match self.peek()? {
            ConditionToken::LParen => {
                self.next();
                let expr = self.parse_or()?;
                if !matches!(self.next(), Some(ConditionToken::RParen)) {
                    return None;
                }
                Some(expr)
            }
            ConditionToken::Ident(_) => self.parse_predicate(),
            _ => None,
        }
    }

    fn parse_predicate(&mut self) -> Option<ConditionExpr> {
        let field = match self.next()? {
            ConditionToken::Ident(s) => s,
            _ => return None,
        };

        match self.peek() {
            Some(ConditionToken::Eq) => {
                self.next();
                let value = self.parse_value()?;
                Some(ConditionExpr::Predicate {
                    field,
                    op: Some(ConditionOp::Eq),
                    value: Some(value),
                })
            }
            Some(ConditionToken::Match) => {
                self.next();
                let value = self.parse_value()?;
                Some(ConditionExpr::Predicate {
                    field,
                    op: Some(ConditionOp::Match),
                    value: Some(value),
                })
            }
            _ => Some(ConditionExpr::Predicate {
                field,
                op: None,
                value: None,
            }),
        }
    }

    fn parse_value(&mut self) -> Option<String> {
        match self.next()? {
            ConditionToken::StringLit(s) => Some(s),
            ConditionToken::Ident(s) => Some(s),
            ConditionToken::Bool(b) => Some(if b { "true".to_string() } else { "false".to_string() }),
            _ => None,
        }
    }

    fn peek(&self) -> Option<&ConditionToken> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<ConditionToken> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }
}

fn tokenize_condition(condition: &str) -> Option<Vec<ConditionToken>> {
    let chars: Vec<char> = condition.chars().collect();
    let mut i = 0usize;
    let mut out = Vec::new();

    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        match ch {
            '(' => {
                out.push(ConditionToken::LParen);
                i += 1;
            }
            ')' => {
                out.push(ConditionToken::RParen);
                i += 1;
            }
            '=' => {
                if i + 1 >= chars.len() {
                    return None;
                }
                if chars[i + 1] == '=' {
                    out.push(ConditionToken::Eq);
                    i += 2;
                } else if chars[i + 1] == '~' {
                    out.push(ConditionToken::Match);
                    i += 2;
                } else {
                    return None;
                }
            }
            '\'' | '"' => {
                let quote = ch;
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }
                if i >= chars.len() {
                    return None;
                }
                let value: String = chars[start..i].iter().collect();
                out.push(ConditionToken::StringLit(value));
                i += 1;
            }
            _ => {
                let start = i;
                while i < chars.len() {
                    let c = chars[i];
                    if c.is_whitespace() || c == '(' || c == ')' || c == '=' {
                        break;
                    }
                    i += 1;
                }
                if start == i {
                    return None;
                }
                let word: String = chars[start..i].iter().collect();
                let lowered = word.to_lowercase();
                match lowered.as_str() {
                    "and" => out.push(ConditionToken::And),
                    "or" => out.push(ConditionToken::Or),
                    "not" => out.push(ConditionToken::Not),
                    "true" => out.push(ConditionToken::Bool(true)),
                    "false" => out.push(ConditionToken::Bool(false)),
                    _ => out.push(ConditionToken::Ident(word)),
                }
            }
        }
    }

    Some(out)
}

fn contains_pattern(value: &str, pattern: &str) -> bool {
    let value_lower = value.to_lowercase();
    pattern.split('|').any(|raw| {
        let mut token = raw.trim().to_lowercase();
        if token.is_empty() {
            return false;
        }

        // Support inline case-insensitive prefix often used in regex-like
        // conditions from migrated configs.
        if let Some(stripped) = token.strip_prefix("(?i)") {
            token = stripped.trim().to_string();
        }

        if token.is_empty() {
            return false;
        }

        // Handle anchored exact-match forms like ^firefox$.
        if token.starts_with('^') && token.ends_with('$') && token.len() >= 2 {
            let exact = &token[1..token.len() - 1];
            return !exact.is_empty() && value_lower == exact;
        }

        // Tolerate partial anchoring by stripping lone anchors.
        let token = token.trim_start_matches('^').trim_end_matches('$');
        !token.is_empty() && value_lower.contains(token)
    })
}

impl WindowContext {
    /// Create a new window context
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if condition matches this window context
    /// Conditions are in the form:
    ///   - "wm_class =~ 'Firefox'" or "wm_name =~ 'Chrome'"
    ///   - "keyboard_type =~ 'IBM'" or "keyboard_type =~ 'IBM, Chromebook'"
    ///   - "settings.Enter2Ent_Cmd" or "settings.Caps2Esc_Cmd"
    pub fn matches_condition(&self, condition: &str) -> bool {
        let condition = condition.trim();
        let tokens = match tokenize_condition(condition) {
            Some(tokens) => tokens,
            None => return false,
        };
        let expr = match ConditionParser::new(tokens).parse() {
            Some(expr) => expr,
            None => return false,
        };
        self.eval_expr(&expr)
    }

    fn eval_expr(&self, expr: &ConditionExpr) -> bool {
        match expr {
            ConditionExpr::And(left, right) => self.eval_expr(left) && self.eval_expr(right),
            ConditionExpr::Or(left, right) => self.eval_expr(left) || self.eval_expr(right),
            ConditionExpr::Not(inner) => !self.eval_expr(inner),
            ConditionExpr::Predicate { field, op, value } => self.eval_predicate(field, *op, value.as_deref()),
        }
    }

    fn eval_predicate(&self, field: &str, op: Option<ConditionOp>, value: Option<&str>) -> bool {
        match op {
            None => self.eval_boolean_field(field),
            Some(ConditionOp::Eq) => self.eval_equals(field, value.unwrap_or_default()),
            Some(ConditionOp::Match) => self.eval_match(field, value.unwrap_or_default()),
        }
    }

    fn eval_boolean_field(&self, field: &str) -> bool {
        if let Some(setting_name) = field.strip_prefix("settings.") {
            return self.settings.get_bool(setting_name);
        }

        match field.to_lowercase().as_str() {
            "numlock" | "numlk" => self.effective_numlock_on(),
            "capslock" | "capslk" => self.capslock_on,
            _ => false,
        }
    }

    fn eval_equals(&self, field: &str, expected: &str) -> bool {
        let expected_lower = expected.to_lowercase();

        if let Some(setting_name) = field.strip_prefix("settings.") {
            let expected_bool = matches!(expected_lower.as_str(), "true" | "1" | "yes" | "on");
            return self.settings.get_bool(setting_name) == expected_bool;
        }

        match field.to_lowercase().as_str() {
            "wm_class" => self
                .wm_class
                .as_ref()
                .map(|v| v.eq_ignore_ascii_case(expected))
                .unwrap_or(false),
            "wm_name" => self
                .wm_name
                .as_ref()
                .map(|v| v.eq_ignore_ascii_case(expected))
                .unwrap_or(false),
            "device_name" | "devn" => self
                .device_name
                .as_ref()
                .map(|v| v.eq_ignore_ascii_case(expected))
                .unwrap_or(false),
            "numlock" | "numlk" => self.effective_numlock_on() == matches!(expected_lower.as_str(), "true" | "1" | "yes" | "on"),
            "capslock" | "capslk" => self.capslock_on == matches!(expected_lower.as_str(), "true" | "1" | "yes" | "on"),
            "keyboard_type" => self
                .keyboard_type
                .map(|kb| kb.as_str().eq_ignore_ascii_case(expected))
                .unwrap_or(false),
            _ => false,
        }
    }

    fn effective_numlock_on(&self) -> bool {
        self.numlock_on || self.settings.get_bool("forced_numpad")
    }

    fn eval_match(&self, field: &str, pattern: &str) -> bool {
        match field.to_lowercase().as_str() {
            "wm_class" => self
                .wm_class
                .as_ref()
                .map(|v| contains_pattern(v, pattern))
                .unwrap_or(false),
            "wm_name" => self
                .wm_name
                .as_ref()
                .map(|v| contains_pattern(v, pattern))
                .unwrap_or(false),
            "device_name" | "devn" => self
                .device_name
                .as_ref()
                .map(|v| contains_pattern(v, pattern))
                .unwrap_or(false),
            "keyboard_type" => self
                .keyboard_type
                .map(|kb| kb.matches(pattern))
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Update window context from class and name
    pub fn update(&mut self, wm_class: Option<String>, wm_name: Option<String>) {
        self.wm_class = wm_class;
        self.wm_name = wm_name;
    }

    /// Update event source device name
    pub fn set_device_name(&mut self, device_name: Option<String>) {
        self.device_name = device_name;
    }

    /// Update lock-state flags
    pub fn set_lock_states(&mut self, numlock_on: bool, capslock_on: bool) {
        self.numlock_on = numlock_on;
        self.capslock_on = capslock_on;
    }

    /// Update keyboard type
    pub fn set_keyboard_type(&mut self, kb_type: crate::input::KeyboardType) {
        self.keyboard_type = Some(kb_type);
    }

    /// Clear keyboard type
    pub fn clear_keyboard_type(&mut self) {
        self.keyboard_type = None;
    }
    
    /// Update settings
    pub fn set_settings(&mut self, settings: crate::settings::Settings) {
        self.settings = settings;
    }
    
    /// Get a reference to settings
    pub fn settings(&self) -> &crate::settings::Settings {
        &self.settings
    }
}

/// Keymap stack for nested keymap support
#[derive(Debug, Clone, Default)]
pub struct KeymapStack {
    /// Stack of keymap names
    pub stack: Vec<String>,
    /// When this keymap was entered (for timeout)
    pub timeout_start: Option<Instant>,
    /// Current active hints
    pub active_hints: Vec<ComboHint>,
}

impl KeymapStack {
    /// Push a keymap onto the stack
    fn push(&mut self, name: String) {
        self.stack.push(name);
    }

    /// Pop the current keymap
    fn pop(&mut self) -> Option<String> {
        self.stack.pop()
    }

    /// Get the current (top) keymap
    fn current(&self) -> Option<&String> {
        self.stack.last()
    }

    /// Clear all keymaps
    fn clear(&mut self) {
        self.stack.clear();
        self.timeout_start = None;
        self.active_hints.clear();
    }

    /// Check if we should timeout
    fn should_timeout(&self, timeout: Duration) -> bool {
        self.timeout_start
            .map(|t| t.elapsed() >= timeout)
            .unwrap_or(false)
    }
}

/// Repeat cache with modifier state tracking
#[derive(Debug, Clone)]
struct RepeatCache {
    key: Key,
    result: TransformResult,
    modifier_snapshot: Vec<Key>,
    timestamp: Instant,
}

impl RepeatCache {
    /// Create a new repeat cache entry
    fn new(key: Key, result: TransformResult, modifier_snapshot: Vec<Key>) -> Self {
        Self {
            key,
            result,
            modifier_snapshot,
            timestamp: Instant::now(),
        }
    }

    /// Check if cache is valid for this key and current modifier state
    fn is_valid(&self, key: Key, current_modifiers: &[Key]) -> bool {
        self.key == key && self.modifier_snapshot.as_slice() == current_modifiers
    }
}

/// Pure Rust transform engine
///
/// This contains all the logic from the Python transform layer
/// but implemented in pure Rust for maximum performance.
#[cfg(feature = "pure-rust")]
pub struct TransformEngine {
    config: TransformConfig,
    keystore: Arc<RwLock<Keystore>>,
    repeat_cache: Option<RepeatCache>,
    /// Window context for conditional evaluation
    window_context: Arc<RwLock<WindowContext>>,
    /// Optional window context provider for tracking active window
    window_manager: Option<Box<dyn WindowContextProvider>>,
    /// Multipurpose manager for tap/hold keys
    multipurpose_manager: MultipurposeManager,
    /// Keymap stack for nested keymaps
    keymap_stack: KeymapStack,
    /// Current hint state
    escape_next: bool,
    /// Current mark value
    mark: Option<bool>,
    /// Whether suspend mode is active
    suspend_mode: bool,
    /// Last time suspend key was pressed (for double-tap detection)
    last_suspend_press: Option<Instant>,
    /// Track combos that have been matched on Press to prevent duplicate matches on Release
    /// Stores (modifier_keys, output_key) tuples
    active_combos: HashSet<(Vec<Key>, Key)>,
    /// Dead key state for accent composition
    deadkeys: DeadKeyState,
}

#[cfg(feature = "pure-rust")]
impl TransformEngine {
    /// Create a new transform engine with given configuration
    pub fn new(config: TransformConfig) -> Self {
        let timeout = config.multipurpose_timeout.unwrap_or(200);
        let mut multipurpose_manager = MultipurposeManager::with_timeout(timeout);
        
        // Load multipurpose modmaps from config
        for multimodmap in &config.multimodmaps {
            multipurpose_manager.add_modmap(multimodmap.clone());
        }
        
        // Load settings from default location
        let settings = crate::settings::Settings::load_default()
            .unwrap_or_else(|_| crate::settings::Settings::new());
        
        let mut window_context = WindowContext::new();
        window_context.set_settings(settings);
        
        Self {
            config,
            keystore: Arc::new(RwLock::new(Keystore::new())),
            repeat_cache: None,
            window_context: Arc::new(RwLock::new(window_context)),
            window_manager: None,
            multipurpose_manager,
            keymap_stack: KeymapStack::default(),
            escape_next: false,
            mark: None,
            suspend_mode: false,
            last_suspend_press: None,
            active_combos: HashSet::new(),
            deadkeys: DeadKeyState::default(),
        }
    }

    /// Create a new transform engine with window context provider
    pub fn with_window_manager(
        config: TransformConfig,
        window_manager: Option<Box<dyn WindowContextProvider>>,
    ) -> Self {
        let timeout = config.multipurpose_timeout.unwrap_or(200);
        let mut multipurpose_manager = MultipurposeManager::with_timeout(timeout);
        
        // Load multipurpose modmaps from config
        for multimodmap in &config.multimodmaps {
            multipurpose_manager.add_modmap(multimodmap.clone());
        }
        
        // Load settings from default location
        let settings = crate::settings::Settings::load_default()
            .unwrap_or_else(|_| crate::settings::Settings::new());
        
        let mut window_context = WindowContext::new();
        window_context.set_settings(settings);
        
        Self {
            config,
            keystore: Arc::new(RwLock::new(Keystore::new())),
            repeat_cache: None,
            window_context: Arc::new(RwLock::new(window_context)),
            window_manager,
            multipurpose_manager,
            keymap_stack: KeymapStack::default(),
            escape_next: false,
            mark: None,
            suspend_mode: false,
            last_suspend_press: None,
            active_combos: HashSet::new(),
            deadkeys: DeadKeyState::default(),
        }
    }

    /// Add a multipurpose modmap entry to the engine
    pub fn add_multipurpose(&mut self, trigger: Key, tap: Key, hold: Key) {
        use crate::mapping::MultiModmap;
        use std::collections::HashMap;
        
        let mut mappings = HashMap::new();
        mappings.insert(trigger, (tap, hold));
        let modmap = MultiModmap::new("multipurpose", mappings);
        self.multipurpose_manager.add_modmap(modmap);
    }

    fn apply_sequence_side_effects(&mut self, steps: &[ActionStep]) -> Vec<ActionStep> {
        let mut output_steps = Vec::with_capacity(steps.len());
        for step in steps {
            match step {
                ActionStep::SetSetting { name, value } => {
                    self.set_setting(name, *value);
                }
                _ => output_steps.push(step.clone()),
            }
        }
        output_steps
    }

    /// Process a single key event
    ///
    /// This is the main entry point for event processing.
    /// It handles modmap lookup, combo matching, and state updates.
    pub fn process_event(&mut self, key: Key, action: Action) -> TransformResult {
        // Handle suspend mode - if active, only the suspend key double-tap can resume
        if self.suspend_mode {
            // Check if this is the suspend key being pressed (for resume)
            if let Some(suspend_key) = self.config.suspend_key {
                if key == suspend_key && action.is_pressed() {
                    // Check for double-tap to resume
                    let now = Instant::now();
                    let timeout = Duration::from_millis(self.config.suspend_timeout.unwrap_or(1000));
                    
                    if let Some(last_press) = self.last_suspend_press {
                        if now.duration_since(last_press) < timeout {
                            // Double-tap detected - resume
                            self.suspend_mode = false;
                            self.last_suspend_press = None;
                            return TransformResult::Suspend;
                        }
                    }
                    // Not a double-tap, update last press time
                    self.last_suspend_press = Some(now);
                }
            }
            
            if action.is_released() {
                // Don't clear suspend_mode on release when we're checking for double-tap
                // The suspend_mode is only toggled by double-tap
            }
            return TransformResult::Suppress;
        }

        // Check for suspend key double-tap (when not suspended)
        if let Some(suspend_key) = self.config.suspend_key {
            if key == suspend_key && action.is_pressed() {
                let now = Instant::now();
                let timeout = Duration::from_millis(self.config.suspend_timeout.unwrap_or(1000));
                
                if let Some(last_press) = self.last_suspend_press {
                    if now.duration_since(last_press) < timeout {
                        // Double-tap detected - suspend
                        self.suspend_mode = true;
                        self.last_suspend_press = None;
                        return TransformResult::Suspend;
                    }
                }
                // Not a double-tap, update last press time and pass through
                self.last_suspend_press = Some(now);
            }
        }

        // Track lock states for condition evaluation (numlock/capslock).
        self.update_lock_state_from_event(key, action);

        // Handle multipurpose (tap/hold) logic first
        if self.multipurpose_manager.has_active() {
            // Check if this is the same key as the active multipurpose
            let is_same_key = self.multipurpose_manager.get_trigger_key() == Some(key);
            
            if is_same_key {
                // This is the multipurpose key being released or repeating
                match action {
                    Action::Release => {
                        // Key released - determine tap vs hold
                        match self.multipurpose_manager.release() {
                            Some(MultipurposeResult::Tap(tap_key)) => {
                                self.keystore.write().update(key, action, None);
                                return TransformResult::Remapped(tap_key);
                            }
                            Some(MultipurposeResult::HoldRelease(hold_key)) => {
                                self.keystore.write().update(hold_key, action, None);
                                return TransformResult::Remapped(hold_key);
                            }
                            None => {
                                // No active multipurpose - fall through to normal processing
                            }
                        }
                    }
                    Action::Repeat => {
                        // Only repeat when the multipurpose key is already in hold state.
                        // While still pending (pre-timeout), repeat must stay suppressed.
                        if self.multipurpose_manager.is_hold_state() {
                            if let Some(hold_key) = self.multipurpose_manager.get_hold_key() {
                                return TransformResult::Remapped(hold_key);
                            }
                        }
                        return TransformResult::Suppress;
                    }
                    Action::Press => {
                        // Shouldn't happen for same key - fall through to normal processing
                    }
                }
            } else {
                // Different key pressed while multipurpose is active
                // This triggers the interrupt behavior (immediate hold)
                if action.is_pressed() {
                    if let Some((hold_key, new_key)) = self.multipurpose_manager.interrupt_with_key(key) {
                        // Output hold key press
                        self.keystore.write().update(hold_key, Action::Press, None);
                        // Continue processing the new key
                        return self.process_interrupting_key(new_key, action);
                    }
                }
            }
        }

        // Check if this key starts a multipurpose sequence
        if action.is_pressed() && self.multipurpose_manager.is_trigger(key) {
            // Multipurpose triggers are for standalone key usage. If another
            // modifier is already held (e.g. RAlt-Enter), skip multipurpose so
            // regular combo/keymap handling can win.
            let has_held_modifier = self
                .keystore
                .read()
                .get_pressed_mods_keys()
                .iter()
                .any(|m| *m != key);
            if has_held_modifier {
                // Fall through to normal processing.
            } else {
            // Check if there's a conditional and evaluate it
            let condition = self.multipurpose_manager.get_conditional(key);
            
            let should_activate = if let Some(cond) = condition {
                // Evaluate the condition against window context
                self.window_context.read().matches_condition(cond)
            } else {
                // No condition, always activate
                true
            };
            
            if should_activate {
                let _started = self.multipurpose_manager.start(key);
                // Suppress the original key until we know tap vs hold
                return TransformResult::Suppress;
            }
            // Condition is false, fall through to normal processing
            }
        }

        // Get current modifier state BEFORE processing this key
        let modifier_snapshot = self.keystore.read().get_modifier_snapshot();

        // Modmap lookup with conditional support (do this BEFORE updating keystore)
        let modmapped_key = self.lookup_modmap(key, &modifier_snapshot);

        // Keep physical modifier identity for combo matching.
        // Modmap output still applies to emitted events, but using remapped
        // modifiers in keystore can make Super-* keymaps miss when Super is
        // remapped to Ctrl by modmap.
        let keystore_key = if Modifier::is_key_modifier(key) {
            key
        } else {
            modmapped_key
        };
        self.keystore.write().update(key, action, Some(keystore_key));

        // Handle hints
        if self.handle_hints(key, &action) {
            return TransformResult::Suppress;
        }

        // Update window context if needed
        // (In production, this would come from Wayland/X11 events)

        // Check for escape_next hint
        if self.escape_next {
            if action.is_pressed() || action.is_repeat() {
                // Let the key through, but don't transform
                self.escape_next = false;
                return TransformResult::Passthrough(key);
            } else {
                self.escape_next = false;
            }
        }

        // Get updated modifier state (modifiers are stored as physical keys in keystore).
        let pressed_mods = self.keystore.read().get_pressed_mods_keys();
        // Also compute logical (modmapped) modifiers for fallback matching, so default
        // Super->Ctrl behavior works unless an explicit Super-* mapping is present.
        let logical_pressed_mods: Vec<Key> = pressed_mods
            .iter()
            .map(|k| self.lookup_modmap(*k, &modifier_snapshot))
            .collect();
        let shift_pressed = logical_pressed_mods
            .iter()
            .any(|k| *k == Key::from(42) || *k == Key::from(54));

        // If a dead key is active, next key press may compose into Unicode.
        if action == Action::Press && self.deadkeys.is_active() {
            if let Some(composed) = self.deadkeys.try_compose(modmapped_key, shift_pressed) {
                return TransformResult::Unicode(composed);
            }
        }

        // Check keymap stack timeout
        if let Some(timeout_val) = self.config.suspend_timeout {
            if self
                .keymap_stack
                .should_timeout(Duration::from_millis(timeout_val))
            {
                self.exit_keymap();
            }
        }

        // Combo matching with precedence:
        // 1) physical modifiers (explicit Super-* exceptions)
        // 2) logical/modmapped modifiers (default Super->Ctrl behavior)
        let mut combo_result = self.find_combo_expanded(&pressed_mods, modmapped_key);
        let mut combo_mods = pressed_mods.clone();
        if matches!(combo_result, ComboMatchResult::NotFound) && logical_pressed_mods != pressed_mods
        {
            let logical_result = self.find_combo_expanded(&logical_pressed_mods, modmapped_key);
            if !matches!(logical_result, ComboMatchResult::NotFound) {
                combo_result = logical_result;
                combo_mods = logical_pressed_mods.clone();
            }
        }

        let result = match combo_result {
            ComboMatchResult::FoundKey(output_key) => {
                if action == Action::Repeat {
                    return TransformResult::Suppress;
                }
                // Check if this is a release of a key that was already matched as a combo
                // This prevents duplicate paste events when releasing a key while modifiers are held
                if action == Action::Release {
                    let combo_key = (combo_mods.clone(), output_key);
                    if self.active_combos.contains(&combo_key) {
                        // Already matched this combo on press, just pass through the release
                        // This prevents duplicate output on release
                        // Remove from active_combos since we're done with this combo
                        self.active_combos.remove(&combo_key);
                        return TransformResult::Suppress;
                    }
                }

                // Track this combo as active on Press (but not for modifier-only combos)
                if action == Action::Press {
                    let combo_key = (combo_mods.clone(), output_key);
                    self.active_combos.insert(combo_key);
                }

                // Check if this enters a nested keymap
                if self.is_keymap_entry(output_key) {
                    self.enter_keymap(output_key);
                }
                TransformResult::ComboKey(output_key)
            }
            ComboMatchResult::FoundCombo(combo) => {
                if action == Action::Repeat {
                    return TransformResult::Suppress;
                }
                // Same fix for FoundCombo - prevent duplicate on Release
                if action == Action::Release {
                    let combo_key = (combo_mods.clone(), combo.key());
                    if self.active_combos.contains(&combo_key) {
                        self.active_combos.remove(&combo_key);
                        return TransformResult::Suppress;
                    }
                }

                // Track this combo as active on Press
                if action == Action::Press {
                    let combo_key = (combo_mods.clone(), combo.key());
                    self.active_combos.insert(combo_key);
                }

                TransformResult::Combo(combo)
            }
            ComboMatchResult::FoundSequence(steps) => {
                if action == Action::Press {
                    let output_steps = self.apply_sequence_side_effects(&steps);
                    if output_steps.is_empty() {
                        TransformResult::Suppress
                    } else {
                        TransformResult::Sequence(output_steps)
                    }
                } else {
                    TransformResult::Suppress
                }
            }
            ComboMatchResult::FoundHint(hint) => TransformResult::Hint(hint),
            ComboMatchResult::FoundUnicode(codepoint) => {
                if action == Action::Press {
                    if self.deadkeys.activate_from_codepoint(codepoint) {
                        TransformResult::Suppress
                    } else {
                        TransformResult::Unicode(codepoint)
                    }
                } else {
                    TransformResult::Suppress
                }
            }
            ComboMatchResult::FoundText(text) => {
                if action == Action::Press {
                    TransformResult::Text(text)
                } else {
                    TransformResult::Suppress
                }
            }
            ComboMatchResult::NotFound => {
                // No combo match, use modmapped key
                // On Release, clean up any active combos involving this key
                if action == Action::Release {
                    self.active_combos.retain(|(mods, _)| {
                        // Keep only combos whose modifiers don't include this key
                        !mods.contains(&key)
                    });
                }

                if modmapped_key != key {
                    TransformResult::Remapped(modmapped_key)
                } else {
                    TransformResult::Passthrough(key)
                }
            }
        };

        // Update repeat cache for REPEAT events
        if action == Action::Repeat {
            if let Some(cache) = &self.repeat_cache {
                if cache.is_valid(key, &pressed_mods) {
                    return cache.result.clone();
                }
            }
            // Convert modifier_snapshot (u16) to Vec<Key>
            let mod_keys: Vec<Key> = modifier_snapshot
                .iter()
                .map(|&code| Key::from(code))
                .collect();
            self.repeat_cache = Some(RepeatCache::new(key, result.clone(), mod_keys));
        } else {
            self.repeat_cache = None;
        }

        result
    }

    fn update_lock_state_from_event(&mut self, key: Key, action: Action) {
        // Toggle on press events, matching lock-key behavior.
        if action != Action::Press {
            return;
        }

        let mut context = self.window_context.write();
        match key.code() {
            69 => context.numlock_on = !context.numlock_on, // NUMLOCK
            58 => context.capslock_on = !context.capslock_on, // CAPSLOCK
            _ => {}
        }
    }

    /// Process a key that interrupted a multipurpose sequence
    fn process_interrupting_key(&mut self, key: Key, action: Action) -> TransformResult {
        // First output the hold key press
        if let Some(hold_key) = self.multipurpose_manager.get_hold_key() {
            self.keystore.write().update(hold_key, Action::Press, None);
        }
        
        // Now process the interrupting key normally
        self.process_event(key, action)
    }

    /// Check if any multipurpose keys have timed out and should transition to hold
    /// This should be called periodically (e.g., in the event loop)
    pub fn check_multipurpose_timeouts(&mut self) -> Option<(Key, Action)> {
        if self.multipurpose_manager.is_pending_state() {
            if let Some(hold_key) = self.multipurpose_manager.check_timeout() {
                // Keep internal state in sync with emitted hold press.
                self.keystore.write().update(hold_key, Action::Press, None);
                return Some((hold_key, Action::Press));
            }
        }
        None
    }

    /// Check if a key is currently an active multipurpose hold key
    pub fn is_multipurpose_hold_active(&self) -> bool {
        self.multipurpose_manager.is_hold_state()
    }

    /// Look up a key through modmaps with conditional evaluation
    fn lookup_modmap(&self, key: Key, _modifier_snapshot: &[u16]) -> Key {
        // Check conditional modmaps first so specific rules can override defaults.
        let context = self.window_context.read();
        for modmap in self.config.modmaps.iter().skip(1) {
            if let Some(condition) = modmap.conditional() {
                if context.matches_condition(condition) {
                    if let Some(remapped) = modmap.get(key) {
                        return remapped;
                    }
                }
            }
        }

        // Fallback to default modmap.
        if let Some(modmap) = self.config.modmaps.first() {
            if let Some(remapped) = modmap.get(key) {
                return remapped;
            }
        }

        key
    }

    /// Find a matching combo with full modifier expansion
    ///
    /// This implements proper handling of non-specific modifiers.
    /// For example, if user defines "ctrl-a" and presses LEFT_CTRL,
    /// it should match because LEFT_CTRL is a Ctrl modifier.
    fn find_combo_expanded(&self, pressed_mods: &[Key], key: Key) -> ComboMatchResult {
        // Convert pressed keys to modifiers
        let mut pressed_modifiers: Vec<Modifier> = pressed_mods
            .iter()
            .filter_map(|k| Modifier::from_key(*k))
            .collect();

        // Also add the current key if it's a modifier
        if let Some(key_mod) = Modifier::from_key(key) {
            // Check if this modifier is not already in pressed list
            if !pressed_modifiers.contains(&key_mod) {
                pressed_modifiers.push(key_mod);
            }
        }

        let combo = Combo::new(pressed_modifiers.clone(), key);
        
        // Get window context for conditional evaluation
        let window_context = self.window_context.read();

        // Try exact match first
        for keymap in &self.config.keymaps {
            // Check if keymap has a condition and if it matches
            if let Some(condition) = keymap.conditional() {
                if !window_context.matches_condition(condition) {
                    continue; // Skip this keymap - condition doesn't match
                }
            }

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

        // Try with non-specific modifier expansion
        // For each modifier in the combo, try replacing with specific variants
        let expansion_attempts = self.expand_modifiers(&combo);

        for expanded_combo in expansion_attempts {
            for keymap in &self.config.keymaps {
                // Check if keymap has a condition and if it matches
                if let Some(condition) = keymap.conditional() {
                    if !window_context.matches_condition(condition) {
                        continue; // Skip this keymap - condition doesn't match
                    }
                }

                if let Some(value) = keymap.get(&expanded_combo) {
                    return match value {
                        KeymapValue::Key(k) => ComboMatchResult::FoundKey(*k),
                        KeymapValue::Combo(c) => ComboMatchResult::FoundCombo(c.clone()),
                        KeymapValue::Sequence(steps) => {
                            ComboMatchResult::FoundSequence(steps.clone())
                        }
                        KeymapValue::ComboHint(h) => ComboMatchResult::FoundHint(*h),
                        KeymapValue::Unicode(codepoint) => {
                            ComboMatchResult::FoundUnicode(*codepoint)
                        }
                        KeymapValue::Text(text) => ComboMatchResult::FoundText(text.clone()),
                    };
                }
            }
        }

        ComboMatchResult::NotFound
    }

    /// Expand a combo by replacing non-specific modifiers with specific variants
    ///
    /// For example: [Ctrl, A] becomes:
    /// - [LCtrl, A], [RCtrl, A]
    fn expand_modifiers(&self, combo: &Combo) -> Vec<Combo> {
        let mut expansions = Vec::new();

        // For each modifier in the combo that could be non-specific...
        for (i, modifier) in combo.modifiers().iter().enumerate() {
            if !modifier.is_specific() {
                // Create variants with this modifier replaced
                let specific_left = modifier.to_left();
                let specific_right = modifier.to_right();

                if let Some(left) = specific_left {
                    let mut new_mods: Vec<Modifier> = combo.modifiers().to_vec();
                    new_mods[i] = left;
                    expansions.push(Combo::new(new_mods, combo.key()));
                }

                if let Some(right) = specific_right {
                    let mut new_mods: Vec<Modifier> = combo.modifiers().to_vec();
                    new_mods[i] = right;
                    expansions.push(Combo::new(new_mods, combo.key()));
                }
            }
        }

        expansions
    }

    /// Handle special hints
    fn handle_hints(&mut self, key: Key, action: &Action) -> bool {
        // Check for SetMark hints
        if let Some(name) = self.get_keymap_name_for_key(key) {
            if name == "set_mark" || name == "mark" {
                match action {
                    Action::Press | Action::Repeat => {
                        self.mark = Some(true);
                        return true;
                    }
                    Action::Release => {
                        self.mark = None;
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if a key is a keymap entry (for nesting)
    fn is_keymap_entry(&self, key: Key) -> bool {
        // Check if any keymap has this as a target
        for keymap in &self.config.keymaps {
            for (_combo, value) in keymap.mappings() {
                if let KeymapValue::Key(k) = value {
                    if *k == key {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get the keymap name that contains this key
    fn get_keymap_name_for_key(&self, key: Key) -> Option<String> {
        for keymap in &self.config.keymaps {
            for (_combo_str, value) in keymap.mappings() {
                if let KeymapValue::Key(k) = value {
                    if *k == key {
                        return Some(keymap.name().to_string());
                    }
                }
            }
        }
        None
    }

    /// Enter a nested keymap
    fn enter_keymap(&mut self, key: Key) {
        // Push the keymap onto stack
        if let Some(name) = self.get_keymap_name_for_key(key) {
            self.keymap_stack.push(name.clone());

            // Set timeout for nested keymap
            if let Some(_timeout) = self.config.suspend_timeout {
                self.keymap_stack.timeout_start = Some(Instant::now());
            }
        }
    }

    /// Exit the current nested keymap
    fn exit_keymap(&mut self) {
        self.keymap_stack.pop();
        self.keymap_stack.timeout_start = None;
    }

    /// Update window context
    pub fn update_window_context(&mut self, wm_class: Option<String>, wm_name: Option<String>) {
        let mut context = self.window_context.write();
        context.update(wm_class, wm_name);

        // Clear keymap stack when window changes
        self.keymap_stack.clear();
    }

    /// Set the window context provider
    ///
    /// This allows the engine to periodically query the active window
    /// for conditional modmap evaluation.
    pub fn set_window_manager(&mut self, window_manager: Option<Box<dyn WindowContextProvider>>) {
        self.window_manager = window_manager;
    }

    /// Set current event-source device name for condition evaluation.
    pub fn set_device_name(&mut self, device_name: Option<String>) {
        self.window_context.write().set_device_name(device_name);
    }

    /// Set lock state flags for condition evaluation.
    pub fn set_lock_states(&mut self, numlock_on: bool, capslock_on: bool) {
        self.window_context
            .write()
            .set_lock_states(numlock_on, capslock_on);
    }

    /// Set keyboard type for condition evaluation.
    pub fn set_keyboard_type(&mut self, kb_type: crate::input::KeyboardType) {
        self.window_context.write().set_keyboard_type(kb_type);
    }

    /// Clear keyboard type from condition context.
    pub fn clear_keyboard_type(&mut self) {
        self.window_context.write().clear_keyboard_type();
    }

    /// Update window context from window manager
    ///
    /// This should be called periodically (e.g., every 100ms) to
    /// update window context for conditional modmap evaluation.
    /// Returns true if window context changed (keymap stack cleared).
    pub fn update_from_window_manager(&mut self) -> bool {
        if let Some(ref manager) = self.window_manager {
            match manager.get_active_window() {
                Ok(info) => {
                    fn normalize_window_field(value: Option<String>) -> Option<String> {
                        let v = value?;
                        let trimmed = v.trim();
                        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("(none)") {
                            None
                        } else {
                            Some(trimmed.to_string())
                        }
                    }

                    let new_wm_class = normalize_window_field(info.wm_class);
                    let new_wm_name = normalize_window_field(info.wm_name);

                    // Keep the last stable window context when the provider returns no active
                    // window details (common transient state during focus switches).
                    if new_wm_class.is_none() && new_wm_name.is_none() {
                        return false;
                    }

                    let mut context = self.window_context.write();

                    // Check if window changed
                    let changed =
                        context.wm_class != new_wm_class || context.wm_name != new_wm_name;

                    // Update context
                    context.wm_class = new_wm_class;
                    context.wm_name = new_wm_name;

                    // Clear keymap stack when window changes
                    if changed {
                        self.keymap_stack.clear();
                    }

                    changed
                }
                Err(_) => {
                    // Window query failed, keep current context
                    false
                }
            }
        } else {
            false
        }
    }

    /// Print current window context for debugging
    pub fn print_window_context(&self) {
        let context = self.window_context.read();
        eprintln!(
            "WINDOW: wm_class={:?} wm_name={:?} device_name={:?} keyboard_type={:?} numlock={} capslock={}",
            context.wm_class.as_deref().unwrap_or("(none)"),
            context.wm_name.as_deref().unwrap_or("(none)"),
            context.device_name.as_deref().unwrap_or("(none)"),
            context.keyboard_type,
            context.numlock_on,
            context.capslock_on
        );
    }

    /// Suspend transformation (for suspend_key)
    pub fn suspend(&mut self) {
        self.suspend_mode = true;
    }

    /// Resume transformation
    pub fn resume(&mut self) {
        self.suspend_mode = false;
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.keystore.write().clear();
        self.repeat_cache = None;
        self.keymap_stack.clear();
        self.escape_next = false;
        self.mark = None;
        self.suspend_mode = false;
        self.last_suspend_press = None;
        self.active_combos.clear();
    }

    /// Get keystore for external inspection
    pub fn keystore(&self) -> &Arc<RwLock<Keystore>> {
        &self.keystore
    }

    /// Get current mark value
    pub fn get_mark(&self) -> Option<bool> {
        self.mark
    }
    
    /// Get settings reference
    pub fn settings(&self) -> crate::settings::Settings {
        self.window_context.read().settings.clone()
    }
    
    /// Update settings
    pub fn set_settings(&mut self, settings: crate::settings::Settings) {
        self.window_context.write().set_settings(settings);
    }
    
    /// Reload settings from disk
    pub fn reload_settings(&mut self) -> Result<(), crate::settings::SettingsError> {
        let settings = crate::settings::Settings::load_default()?;
        self.set_settings(settings);
        Ok(())
    }
    
    /// Get a boolean setting value
    pub fn get_setting(&self, name: &str) -> bool {
        self.window_context.read().settings.get_bool(name)
    }
    
    /// Set a boolean setting value
    pub fn set_setting(&mut self, name: &str, value: bool) {
        self.window_context.write().settings.set_bool(name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::{WindowContextProvider, WindowError, WindowInfo};
    use parking_lot::Mutex;
    use std::collections::HashMap;

    struct ScriptedWindowProvider {
        windows: Mutex<Vec<WindowInfo>>,
    }

    impl ScriptedWindowProvider {
        fn new(windows: Vec<WindowInfo>) -> Self {
            Self {
                windows: Mutex::new(windows),
            }
        }
    }

    impl WindowContextProvider for ScriptedWindowProvider {
        fn connect(&mut self) -> Result<(), WindowError> {
            Ok(())
        }

        fn disconnect(&mut self) {}

        fn is_connected(&self) -> bool {
            true
        }

        fn get_active_window(&self) -> Result<WindowInfo, WindowError> {
            let mut guard = self.windows.lock();
            if guard.is_empty() {
                Ok(WindowInfo::new())
            } else {
                Ok(guard.remove(0))
            }
        }
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_config_default() {
        let config = TransformConfig::default();
        assert!(!config.modmaps.is_empty());
        assert!(config.keymaps.is_empty());
        assert!(config.multipurpose_timeout.is_some());
        assert!(config.suspend_timeout.is_some());
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_update_from_window_manager_ignores_transient_none_none() {
        let mut engine = TransformEngine::new(TransformConfig::default());
        engine.update_window_context(Some("kitty".to_string()), Some("terminal".to_string()));
        engine.set_window_manager(Some(Box::new(ScriptedWindowProvider::new(vec![
            WindowInfo::new(),
            WindowInfo::with_details(
                Some("firefox".to_string()),
                Some("Mozilla Firefox".to_string()),
            ),
        ]))));

        // First update is transient empty context and must be ignored.
        let changed = engine.update_from_window_manager();
        assert!(!changed);
        let ctx = engine.window_context.read().clone();
        assert_eq!(ctx.wm_class.as_deref(), Some("kitty"));
        assert_eq!(ctx.wm_name.as_deref(), Some("terminal"));

        // Second update contains real data and should apply.
        let changed = engine.update_from_window_manager();
        assert!(changed);
        let ctx = engine.window_context.read().clone();
        assert_eq!(ctx.wm_class.as_deref(), Some("firefox"));
        assert_eq!(ctx.wm_name.as_deref(), Some("Mozilla Firefox"));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_window_context_matches() {
        let mut ctx = WindowContext::new();
        ctx.wm_class = Some("Firefox".to_string());

        // Should match regex
        assert!(ctx.matches_condition("wm_class =~ 'fire'"));
        assert!(ctx.matches_condition("wm_class =~ 'Firefox'"));

        // Should not match different class
        assert!(!ctx.matches_condition("wm_class =~ 'Chrome'"));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_window_context_matches_inline_case_and_anchor_patterns() {
        let mut ctx = WindowContext::new();
        ctx.wm_class = Some("firefox".to_string());

        assert!(ctx.matches_condition("wm_class =~ '(?i)Firefox'"));
        assert!(ctx.matches_condition("wm_class =~ '(?i)^FIREFOX$'"));
        assert!(!ctx.matches_condition("wm_class =~ '^fire$'"));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_window_context_condition_boolean_and_device_fields() {
        let mut ctx = WindowContext::new();
        ctx.wm_class = Some("Alacritty".to_string());
        ctx.set_device_name(Some("Telink Wireless Gaming Keyboard".to_string()));
        ctx.set_lock_states(false, true);

        assert!(ctx.matches_condition("device_name =~ 'Telink'"));
        assert!(ctx.matches_condition("devn =~ 'Gaming'"));
        assert!(ctx.matches_condition("capslock == true"));
        assert!(ctx.matches_condition("numlk == false"));
        assert!(!ctx.matches_condition("numlock == true"));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_window_context_condition_logical_composition() {
        let mut ctx = WindowContext::new();
        ctx.wm_class = Some("Alacritty".to_string());
        ctx.wm_name = Some("server".to_string());
        ctx.set_device_name(Some("Telink Keyboard".to_string()));
        ctx.set_lock_states(false, false);
        ctx.settings.set_bool("forced_numpad", true);

        assert!(ctx.matches_condition(
            "settings.forced_numpad and (wm_class =~ 'alacritty' or wm_name =~ 'kitty')"
        ));
        assert!(ctx.matches_condition("numlock and device_name =~ 'telink'"));
        assert!(!ctx.matches_condition("not numlock and device_name =~ 'telink'"));
        assert!(!ctx.matches_condition("settings.forced_numpad and not (wm_class =~ 'alacritty')"));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_forced_numpad_forces_numlock_conditions_true() {
        let mut ctx = WindowContext::new();
        ctx.set_lock_states(false, false);
        ctx.settings.set_bool("forced_numpad", true);

        assert!(ctx.matches_condition("numlock"));
        assert!(ctx.matches_condition("numlk == true"));
        assert!(!ctx.matches_condition("not numlk"));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_window_context_keyboard_type() {
        use crate::input::KeyboardType;
        
        let mut ctx = WindowContext::new();
        ctx.set_keyboard_type(KeyboardType::IBM);

        // Should match single keyboard type
        assert!(ctx.matches_condition("keyboard_type =~ 'IBM'"));
        assert!(!ctx.matches_condition("keyboard_type =~ 'Mac'"));

        // Should match in list
        assert!(ctx.matches_condition("keyboard_type =~ 'IBM, Chromebook'"));
        assert!(ctx.matches_condition("keyboard_type =~ 'Mac, Windows, IBM'"));
        assert!(!ctx.matches_condition("keyboard_type =~ 'Mac, Chromebook'"));

        // Test with different keyboard type
        ctx.set_keyboard_type(KeyboardType::Chromebook);
        assert!(ctx.matches_condition("keyboard_type =~ 'Chromebook'"));
        assert!(ctx.matches_condition("keyboard_type =~ 'IBM, Chromebook'"));
        assert!(!ctx.matches_condition("keyboard_type =~ 'IBM'"));

        // Test without keyboard type set
        ctx.clear_keyboard_type();
        assert!(!ctx.matches_condition("keyboard_type =~ 'IBM'"));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_repeat_cache() {
        let cache = RepeatCache::new(
            Key::from(30),
            TransformResult::Passthrough(Key::from(30)),
            vec![Key::from(29)], // Left Ctrl
        );

        // Should be valid with same modifiers
        assert!(cache.is_valid(Key::from(30), &[Key::from(29)]));

        // Should be invalid with different modifiers
        assert!(!cache.is_valid(
            Key::from(30),
            &[Key::from(56)] // Left Alt
        ));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_conditional_modmap_overrides_default_when_condition_matches() {
        let kp1 = Key::from(79);
        let default_end = Key::from(107);
        let forced_digit_1 = Key::from(2);

        let mut default_map = HashMap::new();
        default_map.insert(kp1, default_end);

        let mut forced_map = HashMap::new();
        forced_map.insert(kp1, forced_digit_1);

        let config = TransformConfig {
            modmaps: vec![
                Modmap::new("default", default_map),
                Modmap::with_conditional(
                    "forced_numpad",
                    forced_map,
                    "settings.forced_numpad".to_string(),
                ),
            ],
            ..TransformConfig::default()
        };

        let mut engine = TransformEngine::new(config);
        engine.set_setting("forced_numpad", false);

        // Default path when setting is false.
        let result = engine.process_event(kp1, Action::Press);
        assert_eq!(result, TransformResult::Remapped(default_end));

        // Conditional path should override default.
        engine.set_setting("forced_numpad", true);
        let result = engine.process_event(kp1, Action::Press);
        assert_eq!(result, TransformResult::Remapped(forced_digit_1));
    }

    // Tests for MultipurposeManager integration
    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_with_multipurpose() {
        let config = TransformConfig::default();
        let mut engine = TransformEngine::new(config);
        
        // Add a multipurpose entry (CAPSLOCK -> ESCAPE tap, RIGHT_CTRL hold)
        engine.add_multipurpose(Key::from(58), Key::from(1), Key::from(97));
        
        // Press the multipurpose key
        let result = engine.process_event(Key::from(58), Action::Press);
        assert_eq!(result, TransformResult::Suppress, "Should suppress initial press");
        
        // Release quickly (tap)
        let result = engine.process_event(Key::from(58), Action::Release);
        assert_eq!(result, TransformResult::Remapped(Key::from(1)), "Should output ESCAPE on tap");
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_multipurpose_interrupt() {
        let config = TransformConfig::default();
        let mut engine = TransformEngine::new(config);
        
        // Add a multipurpose entry
        engine.add_multipurpose(Key::from(58), Key::from(1), Key::from(97));
        
        // Press the multipurpose key
        let _ = engine.process_event(Key::from(58), Action::Press);
        
        // Verify we're in pending state before interrupt
        assert!(engine.multipurpose_manager.is_pending_state());
        
        // Press another key (interrupt)
        let _result = engine.process_event(Key::from(30), Action::Press); // 'A' key
        
        // After interrupt, should still be active but in hold state
        assert!(engine.multipurpose_manager.has_active(), "Multipurpose should still be active after interrupt");
        assert!(engine.multipurpose_manager.is_hold_state(), "Multipurpose should be in hold state after interrupt");
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_multipurpose_timeout_check() {
        let config = TransformConfig {
            multipurpose_timeout: Some(10), // 10ms timeout
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);
        
        // Add a multipurpose entry
        engine.add_multipurpose(Key::from(58), Key::from(1), Key::from(97));
        
        // Press the multipurpose key
        let _ = engine.process_event(Key::from(58), Action::Press);
        
        // Wait for timeout
        std::thread::sleep(Duration::from_millis(50));
        
        // Check timeout
        let timeout_result = engine.check_multipurpose_timeouts();
        assert!(timeout_result.is_some(), "Should detect timeout");
        assert_eq!(timeout_result.unwrap().0, Key::from(97), "Should output RIGHT_CTRL");

        // Timeout transition should also update internal modifier state.
        let pressed_mods = engine.keystore.read().get_pressed_mods_keys();
        assert!(
            pressed_mods.contains(&Key::from(97)),
            "RIGHT_CTRL should be tracked as pressed after timeout hold activation"
        );
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_multipurpose_repeat_suppressed_before_hold() {
        let config = TransformConfig::default();
        let mut engine = TransformEngine::new(config);
        engine.add_multipurpose(Key::from(58), Key::from(1), Key::from(97)); // Caps -> Esc/Ctrl

        let press = engine.process_event(Key::from(58), Action::Press);
        let repeat = engine.process_event(Key::from(58), Action::Repeat);

        assert_eq!(press, TransformResult::Suppress);
        assert_eq!(
            repeat,
            TransformResult::Suppress,
            "Repeat before hold transition must not emit hold key"
        );
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_multipurpose_repeat_emits_hold_after_timeout_transition() {
        let config = TransformConfig {
            multipurpose_timeout: Some(10),
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);
        engine.add_multipurpose(Key::from(58), Key::from(1), Key::from(97)); // Caps -> Esc/Ctrl

        let _ = engine.process_event(Key::from(58), Action::Press);
        std::thread::sleep(Duration::from_millis(50));
        let timeout_result = engine.check_multipurpose_timeouts();
        assert!(timeout_result.is_some(), "Expected hold transition after timeout");

        let repeat = engine.process_event(Key::from(58), Action::Repeat);
        assert_eq!(
            repeat,
            TransformResult::Remapped(Key::from(97)),
            "Repeat after hold transition should repeat hold key"
        );
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_transform_engine_is_multipurpose_hold() {
        let config = TransformConfig::default();
        let mut engine = TransformEngine::new(config);
        
        // Initially not in hold state
        assert!(!engine.is_multipurpose_hold_active());
        
        // Add a multipurpose entry
        engine.add_multipurpose(Key::from(58), Key::from(1), Key::from(97));
        
        // Press the multipurpose key
        let _ = engine.process_event(Key::from(58), Action::Press);
        
        // Still not in hold state (pending)
        assert!(!engine.is_multipurpose_hold_active());
        
        // Trigger interrupt to enter hold state
        let _ = engine.process_event(Key::from(30), Action::Press);
        
        // Now should be in hold state
        assert!(engine.is_multipurpose_hold_active());
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_combo_no_duplicate_on_release() {
        use crate::Combo;

        // This test verifies the fix for the double-paste bug
        // where Cmd+V would paste twice because the combo matched
        // on both Press and Release events

        // Get META modifier (Cmd key)
        let meta_mod = Modifier::from_name("META").expect("META modifier should exist");

        let mut keymap = Keymap::new("test");

        // Create a META+V -> V combo (simulating Cmd+V paste)
        let combo = Combo::new(vec![meta_mod], Key::from(47)); // V key
        keymap.insert(combo, KeymapValue::Key(Key::from(47))); // V

        let config = TransformConfig {
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };

        let mut engine = TransformEngine::new(config);

        // First, press META (Cmd/Meta key)
        let meta_key = Key::from(125); // LEFT_META
        let meta_result = engine.process_event(meta_key, Action::Press);
        assert!(matches!(meta_result, TransformResult::Passthrough(_) | TransformResult::Remapped(_)));

        // Now press V with META held
        let v_key = Key::from(47); // V key
        let v_press_result = engine.process_event(v_key, Action::Press);

        // Should find the combo match
        assert!(matches!(v_press_result, TransformResult::ComboKey(_)));

        // Now release V with META still held
        let v_release_result = engine.process_event(v_key, Action::Release);

        // Should NOT find the combo match again.
        // Release may be suppressed (preferred) or passed through/remapped depending on state.
        assert!(matches!(
            v_release_result,
            TransformResult::Suppress | TransformResult::Passthrough(_) | TransformResult::Remapped(_)
        ));
        assert!(!matches!(v_release_result, TransformResult::ComboKey(_)),
            "Release should NOT return ComboKey - this would cause double paste");
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_super_combo_still_matches_when_super_is_modmapped_to_ctrl() {
        use crate::Combo;

        let mut default_map = HashMap::new();
        default_map.insert(Key::from(125), Key::from(29)); // LEFT_META -> LEFT_CTRL

        let meta_mod = Modifier::from_name("META").expect("META modifier should exist");
        let mut keymap = Keymap::new("super-combo");
        keymap.insert(
            Combo::new(vec![meta_mod], Key::from(51)), // Super-comma
            KeymapValue::Key(Key::from(46)),           // emit C key
        );

        let config = TransformConfig {
            modmaps: vec![Modmap::new("default", default_map)],
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        // Modifier key itself is remapped on output.
        let super_press = engine.process_event(Key::from(125), Action::Press);
        assert_eq!(super_press, TransformResult::Remapped(Key::from(29)));

        // Combo must still match Super-comma even though Super is modmapped.
        let comma_press = engine.process_event(Key::from(51), Action::Press);
        assert_eq!(comma_press, TransformResult::ComboKey(Key::from(46)));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_super_alt_combo_falls_back_to_ctrl_alt_when_not_explicitly_overridden() {
        use crate::Combo;

        let mut default_map = HashMap::new();
        default_map.insert(Key::from(125), Key::from(29)); // LEFT_META -> LEFT_CTRL

        let ctrl = Modifier::from_alias("Ctrl").expect("Ctrl modifier should exist");
        let alt = Modifier::from_alias("Alt").expect("Alt modifier should exist");
        let mut keymap = Keymap::new("ctrl-alt-fallback");
        keymap.insert(
            Combo::new(vec![ctrl, alt], Key::from(46)), // Ctrl-Alt-c
            KeymapValue::Key(Key::from(30)),            // emit A key
        );

        let config = TransformConfig {
            modmaps: vec![Modmap::new("default", default_map)],
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        let _ = engine.process_event(Key::from(125), Action::Press); // Super (modmapped to Ctrl)
        let _ = engine.process_event(Key::from(56), Action::Press); // Alt
        let c_press = engine.process_event(Key::from(46), Action::Press); // c

        assert_eq!(c_press, TransformResult::ComboKey(Key::from(30)));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_explicit_super_alt_combo_overrides_ctrl_alt_fallback() {
        use crate::Combo;

        let mut default_map = HashMap::new();
        default_map.insert(Key::from(125), Key::from(29)); // LEFT_META -> LEFT_CTRL

        let ctrl = Modifier::from_alias("Ctrl").expect("Ctrl modifier should exist");
        let alt = Modifier::from_alias("Alt").expect("Alt modifier should exist");
        let meta = Modifier::from_name("META").expect("META modifier should exist");

        let mut keymap = Keymap::new("super-alt-override");
        keymap.insert(
            Combo::new(vec![ctrl, alt.clone()], Key::from(46)), // Ctrl-Alt-c fallback
            KeymapValue::Key(Key::from(30)),                    // emit A
        );
        keymap.insert(
            Combo::new(vec![meta, alt], Key::from(46)), // explicit Super-Alt-c
            KeymapValue::Key(Key::from(48)),            // emit B
        );

        let config = TransformConfig {
            modmaps: vec![Modmap::new("default", default_map)],
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        let _ = engine.process_event(Key::from(125), Action::Press); // Super
        let _ = engine.process_event(Key::from(56), Action::Press); // Alt
        let c_press = engine.process_event(Key::from(46), Action::Press); // c

        assert_eq!(c_press, TransformResult::ComboKey(Key::from(48)));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_dead_key_composition_from_unicode_mapping() {
        use crate::Combo;

        let ctrl = Modifier::from_alias("Ctrl").expect("Ctrl modifier should exist");
        let mut keymap = Keymap::new("deadkey");
        keymap.insert(
            Combo::new(vec![ctrl.clone()], Key::from(18)), // Ctrl-E
            KeymapValue::Unicode(0x00B4),                  // acute dead key
        );

        let config = TransformConfig {
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        // Hold Ctrl and trigger dead key combo.
        let _ = engine.process_event(Key::from(29), Action::Press); // LEFT_CTRL
        let result = engine.process_event(Key::from(18), Action::Press); // E
        assert_eq!(result, TransformResult::Suppress);

        // Release Ctrl so next key is plain.
        let _ = engine.process_event(Key::from(29), Action::Release);

        // Next letter should compose into Unicode.
        let composed = engine.process_event(Key::from(30), Action::Press); // A
        assert_eq!(composed, TransformResult::Unicode('á' as u32));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_dead_key_space_outputs_accent_symbol() {
        use crate::Combo;

        let ctrl = Modifier::from_alias("Ctrl").expect("Ctrl modifier should exist");
        let mut keymap = Keymap::new("deadkey-space");
        keymap.insert(
            Combo::new(vec![ctrl.clone()], Key::from(18)), // Ctrl-E
            KeymapValue::Unicode(0x00B4),                  // acute dead key
        );

        let config = TransformConfig {
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        let _ = engine.process_event(Key::from(29), Action::Press); // LEFT_CTRL
        let _ = engine.process_event(Key::from(18), Action::Press); // E => activate dead key
        let _ = engine.process_event(Key::from(29), Action::Release);

        // Space after dead key emits accent symbol itself.
        let result = engine.process_event(Key::from(57), Action::Press); // SPACE
        assert_eq!(result, TransformResult::Unicode(0x00B4));
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_unicode_not_emitted_on_repeat() {
        use crate::Combo;

        let ctrl = Modifier::from_alias("Ctrl").expect("Ctrl modifier should exist");
        let mut keymap = Keymap::new("unicode-repeat");
        keymap.insert(
            Combo::new(vec![ctrl], Key::from(18)), // Ctrl-E
            KeymapValue::Unicode(0x00E9),
        );

        let config = TransformConfig {
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        let _ = engine.process_event(Key::from(29), Action::Press); // LEFT_CTRL
        let press = engine.process_event(Key::from(18), Action::Press); // E
        let repeat = engine.process_event(Key::from(18), Action::Repeat); // E repeat

        assert_eq!(press, TransformResult::Unicode(0x00E9));
        assert_eq!(repeat, TransformResult::Suppress);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_text_not_emitted_on_repeat_or_release() {
        use crate::Combo;

        let ctrl = Modifier::from_alias("Ctrl").expect("Ctrl modifier should exist");
        let mut keymap = Keymap::new("text-repeat");
        keymap.insert(
            Combo::new(vec![ctrl], Key::from(20)), // Ctrl-T
            KeymapValue::Text("hello".to_string()),
        );

        let config = TransformConfig {
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        let _ = engine.process_event(Key::from(29), Action::Press); // LEFT_CTRL
        let press = engine.process_event(Key::from(20), Action::Press); // T
        let repeat = engine.process_event(Key::from(20), Action::Repeat); // T repeat
        let release = engine.process_event(Key::from(20), Action::Release); // T release

        assert_eq!(press, TransformResult::Text("hello".to_string()));
        assert_eq!(repeat, TransformResult::Suppress);
        assert_eq!(release, TransformResult::Suppress);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_sequence_not_emitted_on_repeat_or_release() {
        use crate::mapping::ActionStep;
        use crate::Combo;

        let ctrl = Modifier::from_alias("Ctrl").expect("Ctrl modifier should exist");
        let mut keymap = Keymap::new("sequence-repeat");
        keymap.insert(
            Combo::new(vec![ctrl], Key::from(20)), // Ctrl-T
            KeymapValue::Sequence(vec![
                ActionStep::Text("a".to_string()),
                ActionStep::DelayMs(50),
                ActionStep::Text("b".to_string()),
            ]),
        );

        let config = TransformConfig {
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        let _ = engine.process_event(Key::from(29), Action::Press); // LEFT_CTRL
        let press = engine.process_event(Key::from(20), Action::Press); // T
        let repeat = engine.process_event(Key::from(20), Action::Repeat); // T repeat
        let release = engine.process_event(Key::from(20), Action::Release); // T release

        assert!(matches!(press, TransformResult::Sequence(_)));
        assert_eq!(repeat, TransformResult::Suppress);
        assert_eq!(release, TransformResult::Suppress);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_combo_not_emitted_on_repeat() {
        use crate::Combo;

        let mut keymap = Keymap::new("combo-repeat");
        keymap.insert(
            Combo::new(vec![], Key::from(87)), // F11
            KeymapValue::Combo(Combo::new(vec![], Key::from(30))), // A
        );

        let config = TransformConfig {
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        let press = engine.process_event(Key::from(87), Action::Press);
        let repeat = engine.process_event(Key::from(87), Action::Repeat);
        let release = engine.process_event(Key::from(87), Action::Release);

        assert!(matches!(press, TransformResult::Combo(_)));
        assert_eq!(repeat, TransformResult::Suppress);
        assert_eq!(release, TransformResult::Suppress);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_key_output_not_emitted_on_repeat() {
        use crate::Combo;

        let mut keymap = Keymap::new("key-repeat");
        keymap.insert(
            Combo::new(vec![], Key::from(88)), // F12
            KeymapValue::Key(Key::from(30)),   // A
        );

        let config = TransformConfig {
            keymaps: vec![keymap],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);

        let press = engine.process_event(Key::from(88), Action::Press);
        let repeat = engine.process_event(Key::from(88), Action::Repeat);
        let release = engine.process_event(Key::from(88), Action::Release);

        assert_eq!(press, TransformResult::ComboKey(Key::from(30)));
        assert_eq!(repeat, TransformResult::Suppress);
        assert_eq!(release, TransformResult::Suppress);
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_sequence_set_setting_side_effect() {
        use crate::mapping::ActionStep;
        use crate::Combo;

        let mut km_toggle = Keymap::new("toggle");
        km_toggle.insert(
            Combo::new(vec![], Key::from(67)), // F9
            KeymapValue::Sequence(vec![
                ActionStep::SetSetting {
                    name: "Enter2Ent_Cmd".to_string(),
                    value: true,
                },
                ActionStep::Text("ON".to_string()),
            ]),
        );
        km_toggle.insert(
            Combo::new(vec![], Key::from(68)), // F10
            KeymapValue::Sequence(vec![ActionStep::SetSetting {
                name: "Enter2Ent_Cmd".to_string(),
                value: false,
            }]),
        );

        let mut km_true_mappings = std::collections::HashMap::new();
        km_true_mappings.insert(
            Combo::new(vec![], Key::from(66)), // F8
            KeymapValue::Text("TRUE".to_string()),
        );
        let km_true = Keymap::with_conditional(
            "when_true",
            km_true_mappings,
            "settings.Enter2Ent_Cmd".to_string(),
        );

        let mut km_false_mappings = std::collections::HashMap::new();
        km_false_mappings.insert(
            Combo::new(vec![], Key::from(66)), // F8
            KeymapValue::Text("FALSE".to_string()),
        );
        let km_false = Keymap::with_conditional(
            "when_false",
            km_false_mappings,
            "not settings.Enter2Ent_Cmd".to_string(),
        );

        let config = TransformConfig {
            keymaps: vec![km_toggle, km_true, km_false],
            ..TransformConfig::default()
        };
        let mut engine = TransformEngine::new(config);
        engine.set_setting("Enter2Ent_Cmd", false);

        let before = engine.process_event(Key::from(66), Action::Press);
        assert_eq!(before, TransformResult::Text("FALSE".to_string()));

        let toggle = engine.process_event(Key::from(67), Action::Press);
        assert!(matches!(toggle, TransformResult::Sequence(_)));
        assert!(engine.get_setting("Enter2Ent_Cmd"));

        let after_true = engine.process_event(Key::from(66), Action::Press);
        assert_eq!(after_true, TransformResult::Text("TRUE".to_string()));

        let reset = engine.process_event(Key::from(68), Action::Press);
        assert_eq!(reset, TransformResult::Suppress);
        assert!(!engine.get_setting("Enter2Ent_Cmd"));

        let after_false = engine.process_event(Key::from(66), Action::Press);
        assert_eq!(after_false, TransformResult::Text("FALSE".to_string()));
    }
}

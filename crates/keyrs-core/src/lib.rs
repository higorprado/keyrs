// Keyrs Core Library
// Core data models for keyboard remapping

pub mod action;
pub mod combo;
pub mod config;
pub mod input;
pub mod key;
pub mod mapping;
pub mod modifier;
pub mod output;
pub mod state;
pub mod transform;
pub mod trigger;
pub mod window;

// Test module for isolating compilation issues
#[cfg(test)]
mod test_minimal;

#[cfg(feature = "pure-rust")]
pub mod settings;

// Event module is available for both pure-rust and python-runtime features
#[cfg(any(feature = "pure-rust", feature = "python-runtime"))]
pub mod event;

pub use action::Action;
pub use combo::{Combo, ComboHint};
pub use config::{
    expand_combo, expand_keymap_entries, parse_combo_string, ComboParseError, ParsedCombo,
};
pub use input::{
    is_emergency_key, is_key_event, is_keyboard, is_virtual_device, matches_device_filter,
    DeviceCapabilities,
};
pub use key::Key;
pub use mapping::{Keymap, KeymapValue, Keystate, Modmap, MultiModmap, MultipurposeManager, MultipurposeResult};
pub use modifier::{Modifier, ModifierError};

#[cfg(feature = "pure-rust")]
pub use settings::{Settings, SettingsError};
pub use output::{
    calculate_combo_actions, CacheData, ComboActionSequence, OutputCache, PressedKeyState,
};
pub use state::Keystore;
pub use transform::combo::{find_combo_match, ComboMatchResult};
pub use transform::util::{
    get_modifier_snapshot, get_pressed_mods, get_pressed_states, get_spent_state_indices,
};
pub use trigger::Trigger;
pub use window::{ActiveWindow, WaylandClient};

#[cfg(feature = "pure-rust")]
pub use event::{EventLoop, EventLoopError, EventLoopResult};

#[cfg(feature = "python-runtime")]
pub use event::hybrid::{EventReader, HybridError, HybridResult, RawInputEvent, TransformResult};

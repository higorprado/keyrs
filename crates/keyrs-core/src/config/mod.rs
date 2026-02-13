// Keyrs Config API - Rust implementation
// High-performance combo parsing and keymap expansion

pub mod combo_parser;
pub mod keymap_expander;

#[cfg(feature = "pure-rust")]
pub mod parser;

pub use combo_parser::{parse_combo_string, ComboParseError, ParsedCombo};
pub use keymap_expander::{expand_combo, expand_keymap_entries};

#[cfg(feature = "pure-rust")]
pub use parser::{Config, ConfigError, KeymapEntry, KeymapOutput, ModmapEntry, MultipurposeEntry};

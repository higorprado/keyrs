// Xwaykeyz Transform Module
// Core transformation logic for keyboard event processing

pub mod cache;
pub mod combo;
pub mod util;

#[cfg(feature = "pure-rust")]
pub mod deadkeys;

#[cfg(feature = "pure-rust")]
pub mod engine;

pub use cache::{ComboKey, KeymapCache};
pub use combo::{find_combo_match, ComboMatchResult};
pub use util::*;

#[cfg(feature = "pure-rust")]
pub use engine::{TransformConfig, TransformEngine, TransformResult};

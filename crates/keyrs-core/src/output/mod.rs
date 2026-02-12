// Xwaykeyz Output Layer
// Virtual keyboard state management and combo calculation

mod cache;
mod combo;
mod state;

#[cfg(feature = "pure-rust")]
mod uinput;

pub use cache::{CacheData, OutputCache};
pub use combo::{calculate_combo_actions, ComboActionSequence};
pub use state::PressedKeyState;

#[cfg(feature = "pure-rust")]
pub use uinput::{TransformResultOutput, UInputError, VirtualDevice};

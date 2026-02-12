//! Window context module
//!
//! This module provides window context detection for Wayland compositors.

mod provider;
mod wayland;
mod wayland_provider;

pub use provider::{ConditionParseError, WindowCondition, WindowContextProvider, WindowError, WindowInfo};
pub use wayland::{ActiveWindow, WaylandClient, ERR_NO_APP_CLASS, ERR_NO_WDW_TITLE};
pub use wayland_provider::WaylandContextProvider;

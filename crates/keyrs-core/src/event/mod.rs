// Keyrs Event Handling
// Pure Rust event loop and processing

pub mod batch;
#[cfg(feature = "pure-rust")]
pub mod r#loop;

#[cfg(feature = "python-runtime")]
pub mod hybrid;

pub use batch::{batch_config, EventBatch};
#[cfg(feature = "pure-rust")]
pub use evdev::InputEvent;
#[cfg(feature = "pure-rust")]
pub use r#loop::{DeviceInfo, EventLoop, EventLoopError, EventLoopResult};

#[cfg(feature = "python-runtime")]
pub use hybrid::{EventReader, HybridError, HybridResult, RawInputEvent, TransformResult};

// Xwaykeyz Input Layer
// Device detection and filtering logic

mod device;
mod event;
mod filter;
pub mod keyboard_type;

pub use device::{is_keyboard, is_virtual_device, DeviceCapabilities};
pub use event::{is_emergency_key, is_key_event};
pub use filter::matches_device_filter;
pub use keyboard_type::{
    detect_keyboard_type, detect_keyboard_type_simple, keyboard_type_matches,
    DeviceInfo as KeyboardDeviceInfo, KeyboardPatterns, KeyboardType,
};

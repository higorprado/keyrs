// Xwaykeyz Input Layer - Device Detection
// Device capability analysis and keyboard detection

use std::collections::HashSet;

/// Device capabilities extracted from evdev device.capabilities()
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    /// Whether the device supports EV_KEY events
    pub has_ev_key: bool,
    /// List of supported key codes (EV_KEY capability codes)
    pub supported_keys: Vec<u16>,
}

impl DeviceCapabilities {
    /// Create a new DeviceCapabilities struct
    pub fn new(has_ev_key: bool, supported_keys: Vec<u16>) -> Self {
        Self {
            has_ev_key,
            supported_keys,
        }
    }

    /// Check if a specific key code is supported
    pub fn supports_key(&self, key_code: u16) -> bool {
        self.supported_keys.contains(&key_code)
    }

    /// Create a HashSet from supported keys for O(1) lookups
    pub fn key_set(&self) -> HashSet<u16> {
        self.supported_keys.iter().copied().collect()
    }
}

// QWERTY row key codes: Q, W, E, R, T, Y
const QWERTY_CODES: &[u16] = &[16, 17, 18, 19, 20, 21];

// Representative A-Z and SPACE codes for keyboard detection
const A_Z_SPACE_CODES: &[u16] = &[57, 30, 44]; // SPACE, A, Z

/// Determine if a device is a keyboard based on its capabilities.
///
/// A device is considered a keyboard if:
/// 1. It supports EV_KEY events
/// 2. All QWERTY row keys (Q, W, E, R, T, Y) are present
/// 3. Representative A-Z keys (A, Z) and SPACE are present
///
/// This matches the Python implementation in devices.py:
/// ```python
/// QWERTY = [Key.Q, Key.W, Key.E, Key.R, Key.T, Key.Y]  # 16, 17, 18, 19, 20, 21
/// A_Z_SPACE = [Key.SPACE, Key.A, Key.Z]  # 57, 30, 44
///
/// qwerty = all(k in supported_keys for k in QWERTY)
/// az = all(k in supported_keys for k in A_Z_SPACE)
/// if qwerty and az:
///     return True
/// ```
pub fn is_keyboard(capabilities: &DeviceCapabilities) -> bool {
    // Must have EV_KEY capability
    if !capabilities.has_ev_key {
        return false;
    }

    // Use HashSet for O(1) lookups instead of O(n) list.contains()
    let key_set: HashSet<u16> = capabilities.key_set();

    // Check all QWERTY keys are present
    let qwerty_present = QWERTY_CODES.iter().all(|code| key_set.contains(code));

    // Check A-Z representative keys and SPACE are present
    let az_present = A_Z_SPACE_CODES.iter().all(|code| key_set.contains(code));

    qwerty_present && az_present
}

/// Check if a device is a virtual device based on its name.
///
/// Virtual devices are created by keyrs itself and should be
/// filtered out to prevent feedback loops.
///
/// # Arguments
/// * `name` - The device name from evdev
/// * `prefix` - The virtual device prefix (e.g., "Keyrs (virtual)")
pub fn is_virtual_device(name: &str, prefix: &str) -> bool {
    name.contains(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_keyboard_caps() -> DeviceCapabilities {
        // Create a keyboard with all standard keys
        let mut keys = vec![
            0, // RESERVED
        ];

        // Add QWERTY row
        keys.extend_from_slice(QWERTY_CODES);

        // Add A-Z representative keys and SPACE
        keys.extend_from_slice(A_Z_SPACE_CODES);

        // Add some other common keys
        keys.extend_from_slice(&[2, 3, 4, 5, 6, 7, 8, 9, 10, 11]); // Numbers
        keys.extend_from_slice(&[14, 15, 28, 29, 42, 56]); // BACKSPACE, TAB, ENTER, CTRLs, SHIFT, ALT
        keys.extend_from_slice(&[59, 60, 61, 62, 63, 64, 65, 66, 67, 68]); // F1-F10

        DeviceCapabilities::new(true, keys)
    }

    fn make_mouse_caps() -> DeviceCapabilities {
        // Mouse has BTN_LEFT, BTN_RIGHT but no letter keys
        DeviceCapabilities::new(
            true,
            vec![272, 273, 274], // BTN_LEFT, BTN_RIGHT, BTN_MIDDLE
        )
    }

    #[test]
    fn test_is_keyboard_with_full_keyboard() {
        let caps = make_keyboard_caps();
        assert!(is_keyboard(&caps));
    }

    #[test]
    fn test_is_keyboard_without_qwerty() {
        // Create caps with A-Z keys but missing QWERTY
        let mut keys = vec![0];
        keys.extend_from_slice(A_Z_SPACE_CODES);
        keys.extend_from_slice(&[30, 31, 32, 33, 34, 35]); // Some other letters

        let caps = DeviceCapabilities::new(true, keys);
        assert!(!is_keyboard(&caps));
    }

    #[test]
    fn test_is_keyboard_without_az() {
        // Create caps with QWERTY but missing A-Z representative keys
        let mut keys = vec![0];
        keys.extend_from_slice(QWERTY_CODES);

        let caps = DeviceCapabilities::new(true, keys);
        assert!(!is_keyboard(&caps));
    }

    #[test]
    fn test_is_keyboard_with_no_ev_key() {
        let caps = DeviceCapabilities::new(false, vec![]);
        assert!(!is_keyboard(&caps));
    }

    #[test]
    fn test_is_keyboard_mouse_device() {
        let caps = make_mouse_caps();
        assert!(!is_keyboard(&caps));
    }

    #[test]
    fn test_is_virtual_device_with_prefix() {
        assert!(is_virtual_device(
            "Keyrs (virtual) keyboard",
            "Keyrs (virtual)"
        ));
    }

    #[test]
    fn test_is_virtual_device_without_prefix() {
        assert!(!is_virtual_device(
            "Logitech USB Keyboard",
            "Keyrs (virtual)"
        ));
    }

    #[test]
    fn test_device_capabilities_supports_key() {
        let caps = DeviceCapabilities::new(true, vec![16, 17, 18, 30, 57]);
        assert!(caps.supports_key(16)); // Q
        assert!(caps.supports_key(30)); // A
        assert!(!caps.supports_key(100)); // Not in list
    }

    #[test]
    fn test_device_capabilities_key_set() {
        let caps = DeviceCapabilities::new(true, vec![16, 17, 18]);
        let set = caps.key_set();
        assert_eq!(set.len(), 3);
        assert!(set.contains(&16));
        assert!(set.contains(&17));
        assert!(set.contains(&18));
    }
}

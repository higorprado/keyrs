// Keyrs Input Layer - Device Filtering
// Device matching logic for autodetection and manual filtering

/// Check if a device matches the given filter criteria.
///
/// This function implements the device filtering logic from DeviceFilter.filter()
/// in devices.py. The filtering logic is:
///
/// 1. If matches are specified, only match devices by path or name
/// 2. If no matches, exclude virtual devices and non-keyboards
///
/// # Arguments
/// * `device_name` - The device name from evdev
/// * `device_path` - The device path (e.g., "/dev/input/event0")
/// * `filter_names` - List of device names/paths to match (empty for autodetect)
/// * `autodetect` - Whether to autodetect keyboards (true when filter_names is empty)
/// * `is_keyboard` - Whether the device is a keyboard (from is_keyboard())
/// * `is_virtual` - Whether the device is a virtual device (from is_virtual_device())
///
/// # Returns
/// * `true` if the device should be used, `false` otherwise
pub fn matches_device_filter(
    device_name: &str,
    device_path: &str,
    filter_names: &[String],
    autodetect: bool,
    is_keyboard: bool,
    is_virtual: bool,
) -> bool {
    // If matches are specified, only match by path or name
    if !filter_names.is_empty() {
        return filter_names
            .iter()
            .any(|match_name| device_path == match_name || device_name == match_name);
    }

    // Autodetect mode: exclude virtual devices
    if is_virtual {
        return false;
    }

    // Autodetect mode: only use keyboard devices
    if autodetect && !is_keyboard {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_by_path() {
        let filter = vec!["/dev/input/event0".to_string()];
        assert!(matches_device_filter(
            "Logitech Keyboard",
            "/dev/input/event0",
            &filter,
            false,
            true,
            false
        ));
    }

    #[test]
    fn test_matches_by_name() {
        let filter = vec!["Logitech Keyboard".to_string()];
        assert!(matches_device_filter(
            "Logitech Keyboard",
            "/dev/input/event5",
            &filter,
            false,
            true,
            false
        ));
    }

    #[test]
    fn test_no_match_when_filtered() {
        let filter = vec!["Specific Device".to_string()];
        assert!(!matches_device_filter(
            "Other Device",
            "/dev/input/event1",
            &filter,
            false,
            true,
            false
        ));
    }

    #[test]
    fn test_autodetect_keyboard() {
        // Empty filter = autodetect mode
        let filter = vec![];
        assert!(matches_device_filter(
            "Generic Keyboard",
            "/dev/input/event0",
            &filter,
            true,
            true,
            false
        ));
    }

    #[test]
    fn test_autodetect_excludes_non_keyboard() {
        let filter = vec![];
        assert!(!matches_device_filter(
            "Generic Mouse",
            "/dev/input/event1",
            &filter,
            true,
            false,
            false
        ));
    }

    #[test]
    fn test_autodetect_excludes_virtual_device() {
        let filter = vec![];
        assert!(!matches_device_filter(
            "Keyrs (virtual) keyboard",
            "/dev/input/event2",
            &filter,
            true,
            true,
            true
        ));
    }

    #[test]
    fn test_explicit_match_includes_virtual() {
        // When explicitly matched by exact name, even virtual devices are included
        let filter = vec!["Keyrs (virtual) keyboard".to_string()];
        assert!(matches_device_filter(
            "Keyrs (virtual) keyboard",
            "/dev/input/event2",
            &filter,
            false,
            true,
            true
        ));
    }

    #[test]
    fn test_empty_filter_with_autodetect_off() {
        // Empty filter but autodetect off = accept all non-virtual
        let filter = vec![];
        assert!(matches_device_filter(
            "Some Device",
            "/dev/input/event0",
            &filter,
            false,
            false,
            false
        ));
    }
}

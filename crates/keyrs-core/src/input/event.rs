// Keyrs Input Layer - Event Processing Utilities
// Event type checking and emergency key detection

/// EV_KEY event type code from evdev.ecodes
pub const EV_KEY: u16 = 0x01;

/// Check if an event is a key event.
///
/// Key events have event.type == ecodes.EV_KEY (0x01)
pub fn is_key_event(event_type: u16) -> bool {
    event_type == EV_KEY
}

/// Check if a key code is the emergency eject key.
///
/// The eject key can be used as an emergency escape mechanism
/// to terminate keyrs if it gets stuck.
///
/// # Arguments
/// * `key_code` - The key code from the event
/// * `eject_key` - The configured eject key code (from key_codes.rs)
///
/// # Returns
/// * `true` if this is the emergency eject key
pub fn is_emergency_key(key_code: u16, eject_key: u16) -> bool {
    key_code == eject_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_key_event_with_ev_key() {
        assert!(is_key_event(EV_KEY));
    }

    #[test]
    fn test_is_key_event_with_other_event() {
        assert!(!is_key_event(0x02)); // EV_REL
        assert!(!is_key_event(0x00)); // EV_SYN
        assert!(!is_key_event(0x04)); // EV_ABS
    }

    #[test]
    fn test_is_emergency_key_match() {
        assert!(is_emergency_key(161, 161)); // EJECTCD
    }

    #[test]
    fn test_is_emergency_key_no_match() {
        assert!(!is_emergency_key(30, 161)); // A key vs EJECTCD
        assert!(!is_emergency_key(161, 30)); // EJECTCD vs A key
    }

    #[test]
    fn test_ev_key_constant() {
        // EV_KEY should be 0x01 as defined in Linux input-event-codes.h
        assert_eq!(EV_KEY, 0x01);
    }
}

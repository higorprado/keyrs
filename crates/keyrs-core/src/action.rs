use std::fmt;

/// Represents the action state of a key event.
///
/// From `evtest` output, the "magic numbers" for assignment to enums:
///   0 == 'released'
///   1 == 'pressed'
///   2 == 'repeated'
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Action {
    Release = 0,
    Press = 1,
    Repeat = 2,
}

impl Action {
    /// Returns true if the action is either PRESS or REPEAT
    pub fn is_pressed(self) -> bool {
        matches!(self, Action::Press | Action::Repeat)
    }

    /// Returns true only if this is a PRESS event (not REPEAT)
    pub fn just_pressed(self) -> bool {
        matches!(self, Action::Press)
    }

    /// Returns true if this is a RELEASE event
    pub fn is_released(self) -> bool {
        matches!(self, Action::Release)
    }

    /// Returns true if this is a REPEAT event
    pub fn is_repeat(self) -> bool {
        matches!(self, Action::Repeat)
    }

    /// Create Action from i32 value (from evdev)
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Action::Release),
            1 => Some(Action::Press),
            2 => Some(Action::Repeat),
            _ => None,
        }
    }

    /// Convert Action to its i32 representation
    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Release => write!(f, "release"),
            Action::Press => write!(f, "press"),
            Action::Repeat => write!(f, "repeat"),
        }
    }
}

// Module-level constants for Python compatibility
pub const PRESS: Action = Action::Press;
pub const RELEASE: Action = Action::Release;
pub const REPEAT: Action = Action::Repeat;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_properties() {
        assert!(Action::Press.is_pressed());
        assert!(Action::Press.just_pressed());
        assert!(!Action::Press.is_released());
        assert!(!Action::Press.is_repeat());

        assert!(Action::Repeat.is_pressed());
        assert!(!Action::Repeat.just_pressed());
        assert!(!Action::Repeat.is_released());
        assert!(Action::Repeat.is_repeat());

        assert!(!Action::Release.is_pressed());
        assert!(!Action::Release.just_pressed());
        assert!(Action::Release.is_released());
        assert!(!Action::Release.is_repeat());
    }

    #[test]
    fn test_action_from_i32() {
        assert_eq!(Action::from_i32(0), Some(Action::Release));
        assert_eq!(Action::from_i32(1), Some(Action::Press));
        assert_eq!(Action::from_i32(2), Some(Action::Repeat));
        assert_eq!(Action::from_i32(3), None);
    }

    #[test]
    fn test_action_to_i32() {
        assert_eq!(Action::Release.to_i32(), 0);
        assert_eq!(Action::Press.to_i32(), 1);
        assert_eq!(Action::Repeat.to_i32(), 2);
    }
}

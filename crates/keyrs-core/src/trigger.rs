use std::fmt;

/// Defines special keymap behaviors.
///
/// The IMMEDIATELY trigger is used in nested keymaps to provide immediate
/// user feedback. Without it, pressing a nested keymap trigger (like Ctrl-x
/// in Emacs) provides no immediate feedback—the system waits silently for
/// the next key. With immediately, the trigger can send an instant response
/// (like sending 'x') while still entering nested mode for subsequent keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Trigger {
    Immediately = 1,
}

impl Trigger {
    /// Create Trigger from i32 value
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Trigger::Immediately),
            _ => None,
        }
    }

    /// Convert Trigger to its i32 representation
    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trigger::Immediately => write!(f, "IMMEDIATELY"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_from_i32() {
        assert_eq!(Trigger::from_i32(1), Some(Trigger::Immediately));
        assert_eq!(Trigger::from_i32(0), None);
        assert_eq!(Trigger::from_i32(2), None);
    }

    #[test]
    fn test_trigger_to_i32() {
        assert_eq!(Trigger::Immediately.to_i32(), 1);
    }
}

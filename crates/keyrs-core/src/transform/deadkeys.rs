use std::time::{Duration, Instant};

use crate::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadKeyKind {
    Acute,
    Grave,
    Tilde,
    Umlaut,
    Circumflex,
}

impl DeadKeyKind {
    pub fn from_codepoint(codepoint: u32) -> Option<Self> {
        match codepoint {
            0x00B4 => Some(Self::Acute),      // ´
            0x0060 => Some(Self::Grave),      // `
            0x007E | 0x02DC => Some(Self::Tilde), // ~ / ˜
            0x00A8 => Some(Self::Umlaut),     // ¨
            0x005E | 0x02C6 => Some(Self::Circumflex), // ^ / ˆ
            _ => None,
        }
    }

    pub fn display_codepoint(self) -> u32 {
        match self {
            Self::Acute => 0x00B4,
            Self::Grave => 0x0060,
            Self::Tilde => 0x007E,
            Self::Umlaut => 0x00A8,
            Self::Circumflex => 0x005E,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ActiveDeadKey {
    kind: DeadKeyKind,
    activated_at: Instant,
}

#[derive(Debug, Clone)]
pub struct DeadKeyState {
    active: Option<ActiveDeadKey>,
    timeout: Duration,
}

impl Default for DeadKeyState {
    fn default() -> Self {
        Self::new(Duration::from_secs(2))
    }
}

impl DeadKeyState {
    pub fn new(timeout: Duration) -> Self {
        Self {
            active: None,
            timeout,
        }
    }

    pub fn activate_from_codepoint(&mut self, codepoint: u32) -> bool {
        if let Some(kind) = DeadKeyKind::from_codepoint(codepoint) {
            self.active = Some(ActiveDeadKey {
                kind,
                activated_at: Instant::now(),
            });
            true
        } else {
            false
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    pub fn clear(&mut self) {
        self.active = None;
    }

    pub fn try_compose(&mut self, key: Key, shift_pressed: bool) -> Option<u32> {
        let active = self.active?;

        if active.activated_at.elapsed() > self.timeout {
            self.clear();
            return None;
        }

        let result = if key.code() == 57 {
            // Space commits the accent character itself.
            Some(active.kind.display_codepoint())
        } else {
            key_to_ascii_letter(key, shift_pressed)
                .and_then(|ch| compose_letter(active.kind, ch))
                .map(|ch| ch as u32)
        };

        // Dead key is always consumed after the next press attempt.
        self.clear();
        result
    }
}

fn key_to_ascii_letter(key: Key, uppercase: bool) -> Option<char> {
    let name = key.to_string();
    if name.len() == 1 {
        let ch = name.chars().next()?;
        if ch.is_ascii_alphabetic() {
            return Some(if uppercase {
                ch.to_ascii_uppercase()
            } else {
                ch.to_ascii_lowercase()
            });
        }
    }
    None
}

fn compose_letter(kind: DeadKeyKind, base: char) -> Option<char> {
    let out = match kind {
        DeadKeyKind::Acute => match base {
            'a' => 'á',
            'e' => 'é',
            'i' => 'í',
            'o' => 'ó',
            'u' => 'ú',
            'y' => 'ý',
            'A' => 'Á',
            'E' => 'É',
            'I' => 'Í',
            'O' => 'Ó',
            'U' => 'Ú',
            'Y' => 'Ý',
            _ => return None,
        },
        DeadKeyKind::Grave => match base {
            'a' => 'à',
            'e' => 'è',
            'i' => 'ì',
            'o' => 'ò',
            'u' => 'ù',
            'A' => 'À',
            'E' => 'È',
            'I' => 'Ì',
            'O' => 'Ò',
            'U' => 'Ù',
            _ => return None,
        },
        DeadKeyKind::Tilde => match base {
            'a' => 'ã',
            'n' => 'ñ',
            'o' => 'õ',
            'A' => 'Ã',
            'N' => 'Ñ',
            'O' => 'Õ',
            _ => return None,
        },
        DeadKeyKind::Umlaut => match base {
            'a' => 'ä',
            'e' => 'ë',
            'i' => 'ï',
            'o' => 'ö',
            'u' => 'ü',
            'y' => 'ÿ',
            'A' => 'Ä',
            'E' => 'Ë',
            'I' => 'Ï',
            'O' => 'Ö',
            'U' => 'Ü',
            _ => return None,
        },
        DeadKeyKind::Circumflex => match base {
            'a' => 'â',
            'e' => 'ê',
            'i' => 'î',
            'o' => 'ô',
            'u' => 'û',
            'A' => 'Â',
            'E' => 'Ê',
            'I' => 'Î',
            'O' => 'Ô',
            'U' => 'Û',
            _ => return None,
        },
    };

    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_key_activation() {
        let mut state = DeadKeyState::default();
        assert!(state.activate_from_codepoint(0x00B4));
        assert!(state.is_active());
        assert!(!state.activate_from_codepoint(0x20AC));
    }

    #[test]
    fn test_compose_acute_lowercase() {
        let mut state = DeadKeyState::default();
        assert!(state.activate_from_codepoint(0x00B4));
        let out = state.try_compose(Key::from(18), false); // E
        assert_eq!(out, Some('é' as u32));
    }

    #[test]
    fn test_compose_tilde_uppercase() {
        let mut state = DeadKeyState::default();
        assert!(state.activate_from_codepoint(0x007E));
        let out = state.try_compose(Key::from(49), true); // N
        assert_eq!(out, Some('Ñ' as u32));
    }

    #[test]
    fn test_dead_key_timeout_clears_state() {
        let mut state = DeadKeyState::new(Duration::from_millis(1));
        assert!(state.activate_from_codepoint(0x00B4));
        std::thread::sleep(Duration::from_millis(5));
        let out = state.try_compose(Key::from(18), false); // E
        assert_eq!(out, None);
        assert!(!state.is_active());
    }
}

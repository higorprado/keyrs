// Xwaykeyz Key Type
// Represents a single key code from Linux input-event-codes.h

use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;

include!(concat!(env!("OUT_DIR"), "/key_codes.rs"));

/// Display name for a key code
pub fn key_name(code: u16) -> &'static str {
    static KEY_NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
    KEY_NAMES
        .get_or_init(|| {
            let mut names = vec!["UNKNOWN"; 0x300];
            names[0] = "RESERVED";
            names[1] = "ESC";
            names[2] = "KEY_1";
            names[3] = "KEY_2";
            names[4] = "KEY_3";
            names[5] = "KEY_4";
            names[6] = "KEY_5";
            names[7] = "KEY_6";
            names[8] = "KEY_7";
            names[9] = "KEY_8";
            names[10] = "KEY_9";
            names[11] = "KEY_0";
            names[12] = "MINUS";
            names[13] = "EQUAL";
            names[14] = "BACKSPACE";
            names[15] = "TAB";
            names[16] = "Q";
            names[17] = "W";
            names[18] = "E";
            names[19] = "R";
            names[20] = "T";
            names[21] = "Y";
            names[22] = "U";
            names[23] = "I";
            names[24] = "O";
            names[25] = "P";
            names[26] = "LEFT_BRACE";
            names[27] = "RIGHT_BRACE";
            names[28] = "ENTER";
            names[29] = "LEFT_CTRL";
            names[30] = "A";
            names[31] = "S";
            names[32] = "D";
            names[33] = "F";
            names[34] = "G";
            names[35] = "H";
            names[36] = "J";
            names[37] = "K";
            names[38] = "L";
            names[39] = "SEMICOLON";
            names[40] = "APOSTROPHE";
            names[41] = "GRAVE";
            names[42] = "LEFT_SHIFT";
            names[43] = "BACKSLASH";
            names[44] = "Z";
            names[45] = "X";
            names[46] = "C";
            names[47] = "V";
            names[48] = "B";
            names[49] = "N";
            names[50] = "M";
            names[51] = "COMMA";
            names[52] = "DOT";
            names[53] = "SLASH";
            names[54] = "RIGHT_SHIFT";
            names[55] = "KPASTERISK";
            names[56] = "LEFT_ALT";
            names[57] = "SPACE";
            names[58] = "CAPSLOCK";
            names[59] = "F1";
            names[60] = "F2";
            names[61] = "F3";
            names[62] = "F4";
            names[63] = "F5";
            names[64] = "F6";
            names[65] = "F7";
            names[66] = "F8";
            names[67] = "F9";
            names[68] = "F10";
            names[69] = "NUMLOCK";
            names[70] = "SCROLLLOCK";
            names[71] = "KP7";
            names[72] = "KP8";
            names[73] = "KP9";
            names[74] = "KPMINUS";
            names[75] = "KP4";
            names[76] = "KP5";
            names[77] = "KP6";
            names[78] = "KPPLUS";
            names[79] = "KP1";
            names[80] = "KP2";
            names[81] = "KP3";
            names[82] = "KP0";
            names[83] = "KPDOT";
            names[85] = "ZENKAKUHANKAKU";
            names[86] = "KEY_102ND";
            names[87] = "F11";
            names[88] = "F12";
            names[89] = "RO";
            names[90] = "KATAKANA";
            names[91] = "HIRAGANA";
            names[92] = "HENKAN";
            names[93] = "KATAKANAHIRAGANA";
            names[94] = "MUHENKAN";
            names[95] = "KPJPCOMMA";
            names[96] = "KPENTER";
            names[97] = "RIGHT_CTRL";
            names[98] = "KPSLASH";
            names[99] = "SYSRQ";
            names[100] = "RIGHT_ALT";
            names[101] = "LINEFEED";
            names[102] = "HOME";
            names[103] = "UP";
            names[104] = "PAGE_UP";
            names[105] = "LEFT";
            names[106] = "RIGHT";
            names[107] = "END";
            names[108] = "DOWN";
            names[109] = "PAGE_DOWN";
            names[110] = "INSERT";
            names[111] = "DELETE";
            names[112] = "MACRO";
            names[113] = "MUTE";
            names[114] = "VOLUMEDOWN";
            names[115] = "VOLUMEUP";
            names[116] = "POWER";
            names[117] = "KPEQUAL";
            names[118] = "KPPLUSMINUS";
            names[119] = "PAUSE";
            names[120] = "SCALE";
            names[121] = "KPCOMMA";
            names[122] = "HANGEUL";
            names[123] = "HANJA";
            names[124] = "YEN";
            names[125] = "LEFT_META";
            names[126] = "RIGHT_META";
            names[127] = "COMPOSE";
            names[128] = "STOP";
            names[129] = "AGAIN";
            names[130] = "PROPS";
            names[131] = "UNDO";
            names[132] = "FRONT";
            names[133] = "COPY";
            names[134] = "OPEN";
            names[135] = "PASTE";
            names[136] = "FIND";
            names[137] = "CUT";
            names[138] = "HELP";
            names[139] = "MENU";
            names[140] = "CALC";
            names[141] = "SETUP";
            names[142] = "SLEEP";
            names[143] = "WAKEUP";
            names[144] = "FILE";
            names[145] = "SENDFILE";
            names[146] = "DELETEFILE";
            names[147] = "XFER";
            names[148] = "PROG1";
            names[149] = "PROG2";
            names[150] = "WWW";
            names[151] = "MSDOS";
            names[152] = "COFFEE";
            names[153] = "DIRECTION";
            names[154] = "CYCLEWINDOWS";
            names[155] = "MAIL";
            names[156] = "BOOKMARKS";
            names[157] = "COMPUTER";
            names[158] = "BACK";
            names[159] = "FORWARD";
            names[160] = "CLOSECD";
            names[161] = "EJECTCD";
            names[162] = "EJECTCLOSECD";
            names[163] = "NEXTSONG";
            names[164] = "PLAYPAUSE";
            names[165] = "PREVIOUSSONG";
            names[166] = "STOPCD";
            names[167] = "RECORD";
            names[168] = "REWIND";
            names[169] = "PHONE";
            names[170] = "ISO";
            names[171] = "CONFIG";
            names[172] = "HOMEPAGE";
            names[173] = "REFRESH";
            names[174] = "EXIT";
            names[175] = "MOVE";
            names[176] = "EDIT";
            names[177] = "SCROLLUP";
            names[178] = "SCROLLDOWN";
            names[179] = "KPLEFTPAREN";
            names[180] = "KPRIGHTPAREN";
            names[181] = "NEW";
            names[182] = "REDO";
            names[183] = "F13";
            names[184] = "F14";
            names[185] = "F15";
            names[186] = "F16";
            names[187] = "F17";
            names[188] = "F18";
            names[189] = "F19";
            names[190] = "F20";
            names[191] = "F21";
            names[192] = "F22";
            names[193] = "F23";
            names[194] = "F24";
            names[200] = "PLAYCD";
            names[201] = "PAUSECD";
            names[202] = "PROG3";
            names[203] = "PROG4";
            names[204] = "DASHBOARD";
            names[205] = "SUSPEND";
            names[206] = "CLOSE";
            names[207] = "PLAY";
            names[208] = "FASTFORWARD";
            names[209] = "BASSBOOST";
            names[210] = "PRINT";
            names[211] = "HP";
            names[212] = "CAMERA";
            names[213] = "SOUND";
            names[214] = "QUESTION";
            names[215] = "EMAIL";
            names[216] = "CHAT";
            names[217] = "SEARCH";
            names[218] = "CONNECT";
            names[219] = "FINANCE";
            names[220] = "SPORT";
            names[221] = "SHOP";
            names[222] = "ALTERASE";
            names[223] = "CANCEL";
            names[224] = "BRIGHTNESSDOWN";
            names[225] = "BRIGHTNESSUP";
            names[226] = "MEDIA";
            names[227] = "SWITCHVIDEOMODE";
            names[228] = "KBDILLUMTOGGLE";
            names[229] = "KBDILLUMDOWN";
            names[230] = "KBDILLUMUP";
            names[231] = "SEND";
            names[232] = "REPLY";
            names[233] = "FORWARDMAIL";
            names[234] = "SAVE";
            names[235] = "DOCUMENTS";
            names[236] = "BATTERY";
            names[237] = "BLUETOOTH";
            names[238] = "WLAN";
            names[239] = "UWB";
            names[240] = "UNKNOWN";
            names[241] = "VIDEO_NEXT";
            names[242] = "VIDEO_PREV";
            names[243] = "BRIGHTNESS_CYCLE";
            names[244] = "BRIGHTNESS_AUTO";
            names[245] = "DISPLAY_OFF";
            names[246] = "WWAN";
            names[247] = "RFKILL";
            names[248] = "MICMUTE";
            names
        })
        .get(code as usize)
        .copied()
        .unwrap_or("UNKNOWN")
}

/// Try to parse a key name to a key code
pub fn key_from_name(name: &str) -> Option<Key> {
    let name_upper = name.to_uppercase();
    static NAME_TO_CODE: OnceLock<Vec<(&'static str, u16)>> = OnceLock::new();
    let map = NAME_TO_CODE.get_or_init(|| {
        vec![
            ("RESERVED", 0),
            ("ESC", 1),
            ("ESCAPE", 1),
            ("KEY_1", 2),
            ("1", 2),
            ("KEY_2", 3),
            ("2", 3),
            ("KEY_3", 4),
            ("3", 4),
            ("KEY_4", 5),
            ("4", 5),
            ("KEY_5", 6),
            ("5", 6),
            ("KEY_6", 7),
            ("6", 7),
            ("KEY_7", 8),
            ("7", 8),
            ("KEY_8", 9),
            ("8", 9),
            ("KEY_9", 10),
            ("9", 10),
            ("KEY_0", 11),
            ("0", 11),
            ("MINUS", 12),
            ("EQUAL", 13),
            ("BACKSPACE", 14),
            ("TAB", 15),
            ("Q", 16),
            ("W", 17),
            ("E", 18),
            ("R", 19),
            ("T", 20),
            ("Y", 21),
            ("U", 22),
            ("I", 23),
            ("O", 24),
            ("P", 25),
            ("LEFT_BRACE", 26),
            ("RIGHT_BRACE", 27),
            ("ENTER", 28),
            ("LEFT_CTRL", 29),
            ("A", 30),
            ("S", 31),
            ("D", 32),
            ("F", 33),
            ("G", 34),
            ("H", 35),
            ("J", 36),
            ("K", 37),
            ("L", 38),
            ("SEMICOLON", 39),
            ("APOSTROPHE", 40),
            ("GRAVE", 41),
            ("LEFT_SHIFT", 42),
            ("BACKSLASH", 43),
            ("Z", 44),
            ("X", 45),
            ("C", 46),
            ("V", 47),
            ("B", 48),
            ("N", 49),
            ("M", 50),
            ("COMMA", 51),
            ("DOT", 52),
            ("SLASH", 53),
            ("RIGHT_SHIFT", 54),
            ("KPASTERISK", 55),
            ("KP7", 71), ("KP8", 72), ("KP9", 73),
            ("KPMINUS", 74),
            ("KP4", 75), ("KP5", 76), ("KP6", 77),
            ("KPPLUS", 78),
            ("KP1", 79), ("KP2", 80), ("KP3", 81),
            ("KP0", 82),
            ("KPDOT", 83),
            ("KPENTER", 96),
            ("KPSLASH", 98),
            ("KPEQUAL", 117),
            ("KPPLUSMINUS", 118),
            ("KPCOMMA", 121),
            ("LEFT_ALT", 56),
            ("SPACE", 57),
            ("CAPSLOCK", 58),
            ("F1", 59),
            ("F2", 60),
            ("F3", 61),
            ("F4", 62),
            ("F5", 63),
            ("F6", 64),
            ("F7", 65),
            ("F8", 66),
            ("F9", 67),
            ("F10", 68),
            ("NUMLOCK", 69),
            ("SCROLLLOCK", 70),
            ("SYSRQ", 99),
            ("PRINT", 99),
            ("PRTSCR", 99),
            ("PAUSE", 119),
            ("RIGHT_CTRL", 97),
            ("RIGHT_ALT", 100),
            ("HOME", 102),
            ("UP", 103),
            ("PAGE_UP", 104),
            ("LEFT", 105),
            ("RIGHT", 106),
            ("END", 107),
            ("DOWN", 108),
            ("PAGE_DOWN", 109),
            ("INSERT", 110),
            ("DELETE", 111),
            ("MUTE", 113),
            ("VOLUMEDOWN", 114),
            ("VOLUMEUP", 115),
            ("LEFT_META", 125),
            ("RIGHT_META", 126),
            ("MENU", 139),
            ("F11", 87),
            ("F12", 88),
            ("F13", 183),
            ("F14", 184),
            ("F15", 185),
            ("F16", 186),
            ("F17", 187),
            ("F18", 188),
            ("F19", 189),
            ("F20", 190),
            ("F21", 191),
            ("F22", 192),
            ("F23", 193),
            ("F24", 194),
            ("PLAYPAUSE", 164),
            ("STOPCD", 166),
            ("PREVIOUSSONG", 165),
            ("NEXTSONG", 163),
        ]
    });
    map.iter()
        .find(|(n, _)| *n == name_upper)
        .map(|(_, code)| Key::from(*code))
}

/// ASCII character to key code mapping
pub fn ascii_to_key(c: char) -> Option<Key> {
    match c {
        ';' => Some(Key::from(39)),  // SEMICOLON
        '\'' => Some(Key::from(40)), // APOSTROPHE
        '=' => Some(Key::from(13)),  // EQUAL
        '-' => Some(Key::from(12)),  // MINUS
        '`' => Some(Key::from(41)),  // GRAVE
        '[' => Some(Key::from(26)),  // LEFT_BRACE
        ']' => Some(Key::from(27)),  // RIGHT_BRACE
        ',' => Some(Key::from(51)),  // COMMA
        '.' => Some(Key::from(52)),  // DOT
        '/' => Some(Key::from(53)),  // SLASH
        ' ' => Some(Key::from(57)),  // SPACE
        '\\' => Some(Key::from(43)), // BACKSLASH
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_from_name() {
        assert_eq!(key_from_name("a"), Some(Key::from(30)));
        assert_eq!(key_from_name("A"), Some(Key::from(30)));
        assert_eq!(key_from_name("ENTER"), Some(Key::from(28)));
        assert_eq!(key_from_name("1"), Some(Key::from(2)));
        assert_eq!(key_from_name("0"), Some(Key::from(11)));
        assert_eq!(key_from_name("PRINT"), Some(Key::from(99)));
        assert_eq!(key_from_name("PAUSE"), Some(Key::from(119)));
    }

    #[test]
    fn test_key_display() {
        assert_eq!(Key::from(30).to_string(), "A");
        assert_eq!(Key::from(28).to_string(), "ENTER");
    }

    #[test]
    fn test_ascii_to_key() {
        assert_eq!(ascii_to_key(';'), Some(Key::from(39)));
        assert_eq!(ascii_to_key(' '), Some(Key::from(57)));
        assert_eq!(ascii_to_key('x'), None);
    }

    #[test]
    fn test_key_equality() {
        let key1 = Key::from(30);
        let key2 = Key::from(30);
        let key3 = Key::from(31);
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_key_ordering() {
        let key1 = Key::from(30);
        let key2 = Key::from(31);
        assert!(key1 < key2);
    }

    #[test]
    fn test_key_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(Key::from(30), "value");
        assert_eq!(map.get(&Key::from(30)), Some(&"value"));
    }
}

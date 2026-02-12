// Xwaykeyz Input Layer - Keyboard Type Detection
// Detects keyboard variants: IBM, Chromebook, Windows, Mac

use std::collections::HashMap;

/// Keyboard type variants supported by keyrs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardType {
    /// IBM-style keyboards (ThinkPad, etc.)
    /// Features: Physical keys for PgUp/PgDown, no search key
    IBM,
    /// Chromebook keyboards
    /// Features: Search key instead of Caps Lock, different function row
    Chromebook,
    /// Standard Windows keyboards
    /// Features: Windows/Super key, standard layout
    Windows,
    /// Apple/Mac keyboards
    /// Features: Command/Option keys, media keys
    Mac,
    /// Unknown/unsupported keyboard type
    Unknown,
}

impl KeyboardType {
    /// Convert string to KeyboardType
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ibm" => Some(KeyboardType::IBM),
            "chromebook" | "chrome" => Some(KeyboardType::Chromebook),
            "windows" | "win" | "pc" => Some(KeyboardType::Windows),
            "mac" | "apple" | "macintosh" => Some(KeyboardType::Mac),
            "unknown" => Some(KeyboardType::Unknown),
            _ => None,
        }
    }

    /// Convert KeyboardType to string
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyboardType::IBM => "IBM",
            KeyboardType::Chromebook => "Chromebook",
            KeyboardType::Windows => "Windows",
            KeyboardType::Mac => "Mac",
            KeyboardType::Unknown => "Unknown",
        }
    }

    /// Check if this keyboard type matches a condition string
    /// Supports single types or comma-separated lists
    pub fn matches(&self, condition: &str) -> bool {
        condition
            .split(',')
            .map(|s| s.trim())
            .filter_map(KeyboardType::from_str)
            .any(|t| t == *self)
    }
}

impl std::fmt::Display for KeyboardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Device identification information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device name from evdev
    pub name: String,
    /// Vendor ID (USB VID)
    pub vendor_id: Option<u16>,
    /// Product ID (USB PID)
    pub product_id: Option<u16>,
    /// Physical path
    pub phys: Option<String>,
}

impl DeviceInfo {
    /// Create new DeviceInfo
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vendor_id: None,
            product_id: None,
            phys: None,
        }
    }

    /// Set vendor ID
    pub fn with_vendor_id(mut self, vid: u16) -> Self {
        self.vendor_id = Some(vid);
        self
    }

    /// Set product ID
    pub fn with_product_id(mut self, pid: u16) -> Self {
        self.product_id = Some(pid);
        self
    }

    /// Set physical path
    pub fn with_phys(mut self, phys: impl Into<String>) -> Self {
        self.phys = Some(phys.into());
        self
    }
}

/// Keyboard detection patterns
pub struct KeyboardPatterns {
    /// Name patterns for IBM keyboards
    ibm_patterns: Vec<&'static str>,
    /// Name patterns for Chromebook keyboards
    chromebook_patterns: Vec<&'static str>,
    /// Name patterns for Windows keyboards
    windows_patterns: Vec<&'static str>,
    /// Name patterns for Mac keyboards
    mac_patterns: Vec<&'static str>,
    /// Vendor ID to keyboard type mappings
    vendor_mappings: HashMap<u16, KeyboardType>,
}

impl Default for KeyboardPatterns {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardPatterns {
    /// Create new keyboard patterns with default values
    pub fn new() -> Self {
        let mut vendor_mappings = HashMap::new();
        // Apple vendor ID
        vendor_mappings.insert(0x05ac, KeyboardType::Mac);
        // Logitech (often Windows keyboards)
        vendor_mappings.insert(0x046d, KeyboardType::Windows);
        // Lenovo (often IBM-style)
        vendor_mappings.insert(0x17ef, KeyboardType::IBM);
        // Google (Chromebooks)
        vendor_mappings.insert(0x18d1, KeyboardType::Chromebook);
        vendor_mappings.insert(0x00f3, KeyboardType::Chromebook);

        Self {
            ibm_patterns: vec![
                "thinkpad",
                "trackpoint",
                "lenovo",
                "ibm",
                "compact usb keyboard",  // IBM compact keyboards
            ],
            chromebook_patterns: vec![
                "chromebook",
                "chrome",
                "cros",
                "pixelbook",
                "pixel slate",
            ],
            windows_patterns: vec![
                "windows",
                "microsoft",
                "logitech",
                "dell",
                "hp",
                "telink",
                "wireless gaming keyboard",
                "cooler master",
                "razer",
                "corsair",
                "steelseries",
            ],
            mac_patterns: vec![
                "apple",
                "magic keyboard",
                "macbook",
                "imac",
            ],
            vendor_mappings,
        }
    }

    /// Add custom name patterns
    pub fn with_ibm_patterns(mut self, patterns: Vec<&'static str>) -> Self {
        self.ibm_patterns.extend(patterns);
        self
    }

    pub fn with_chromebook_patterns(mut self, patterns: Vec<&'static str>) -> Self {
        self.chromebook_patterns.extend(patterns);
        self
    }

    pub fn with_windows_patterns(mut self, patterns: Vec<&'static str>) -> Self {
        self.windows_patterns.extend(patterns);
        self
    }

    pub fn with_mac_patterns(mut self, patterns: Vec<&'static str>) -> Self {
        self.mac_patterns.extend(patterns);
        self
    }

    /// Add vendor ID mapping
    pub fn add_vendor_mapping(&mut self, vid: u16, kb_type: KeyboardType) {
        self.vendor_mappings.insert(vid, kb_type);
    }
}

/// Detect keyboard type from device information
pub fn detect_keyboard_type(device: &DeviceInfo, patterns: &KeyboardPatterns) -> KeyboardType {
    let name_lower = device.name.to_lowercase();

    // First check vendor ID (most reliable)
    if let Some(vid) = device.vendor_id {
        if let Some(kb_type) = patterns.vendor_mappings.get(&vid) {
            return *kb_type;
        }
    }

    // Check name patterns
    for pattern in &patterns.mac_patterns {
        if name_lower.contains(pattern) {
            return KeyboardType::Mac;
        }
    }

    for pattern in &patterns.chromebook_patterns {
        if name_lower.contains(pattern) {
            return KeyboardType::Chromebook;
        }
    }

    for pattern in &patterns.ibm_patterns {
        if name_lower.contains(pattern) {
            return KeyboardType::IBM;
        }
    }

    for pattern in &patterns.windows_patterns {
        if name_lower.contains(pattern) {
            return KeyboardType::Windows;
        }
    }

    // Check physical path for Chromebooks (often contain specific paths)
    if let Some(phys) = &device.phys {
        let phys_lower = phys.to_lowercase();
        if phys_lower.contains("cros") || phys_lower.contains("chrome") {
            return KeyboardType::Chromebook;
        }
    }

    KeyboardType::Unknown
}

/// Simple detection using default patterns
pub fn detect_keyboard_type_simple(device: &DeviceInfo) -> KeyboardType {
    detect_keyboard_type(device, &KeyboardPatterns::new())
}

/// Check if a keyboard type condition matches
pub fn keyboard_type_matches(kb_type: KeyboardType, condition: &str) -> bool {
    kb_type.matches(condition)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_type_from_str() {
        assert_eq!(KeyboardType::from_str("IBM"), Some(KeyboardType::IBM));
        assert_eq!(KeyboardType::from_str("ibm"), Some(KeyboardType::IBM));
        assert_eq!(KeyboardType::from_str("Chromebook"), Some(KeyboardType::Chromebook));
        assert_eq!(KeyboardType::from_str("chrome"), Some(KeyboardType::Chromebook));
        assert_eq!(KeyboardType::from_str("Windows"), Some(KeyboardType::Windows));
        assert_eq!(KeyboardType::from_str("win"), Some(KeyboardType::Windows));
        assert_eq!(KeyboardType::from_str("Mac"), Some(KeyboardType::Mac));
        assert_eq!(KeyboardType::from_str("apple"), Some(KeyboardType::Mac));
        assert_eq!(KeyboardType::from_str("unknown"), Some(KeyboardType::Unknown));
        assert_eq!(KeyboardType::from_str("invalid"), None);
    }

    #[test]
    fn test_keyboard_type_as_str() {
        assert_eq!(KeyboardType::IBM.as_str(), "IBM");
        assert_eq!(KeyboardType::Chromebook.as_str(), "Chromebook");
        assert_eq!(KeyboardType::Windows.as_str(), "Windows");
        assert_eq!(KeyboardType::Mac.as_str(), "Mac");
        assert_eq!(KeyboardType::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_keyboard_type_matches_single() {
        assert!(KeyboardType::IBM.matches("IBM"));
        assert!(!KeyboardType::IBM.matches("Mac"));
    }

    #[test]
    fn test_keyboard_type_matches_list() {
        assert!(KeyboardType::IBM.matches("IBM, Chromebook, Windows"));
        assert!(KeyboardType::Chromebook.matches("IBM, Chromebook, Windows"));
        assert!(!KeyboardType::Mac.matches("IBM, Chromebook, Windows"));
    }

    #[test]
    fn test_keyboard_type_display() {
        assert_eq!(format!("{}", KeyboardType::IBM), "IBM");
        assert_eq!(format!("{}", KeyboardType::Chromebook), "Chromebook");
    }

    #[test]
    fn test_device_info_creation() {
        let device = DeviceInfo::new("Test Keyboard")
            .with_vendor_id(0x1234)
            .with_product_id(0x5678)
            .with_phys("usb-0000:00:14.0-2/input0");

        assert_eq!(device.name, "Test Keyboard");
        assert_eq!(device.vendor_id, Some(0x1234));
        assert_eq!(device.product_id, Some(0x5678));
        assert_eq!(device.phys, Some("usb-0000:00:14.0-2/input0".to_string()));
    }

    #[test]
    fn test_detect_ibm_by_name() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("Lenovo ThinkPad Compact USB Keyboard");
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::IBM);
    }

    #[test]
    fn test_detect_chromebook_by_name() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("Google Chromebook Keyboard");
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::Chromebook);
    }

    #[test]
    fn test_detect_windows_by_name() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("Logitech USB Keyboard");
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::Windows);
    }

    #[test]
    fn test_detect_telink_by_name_as_windows() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("Telink Wireless Gaming Keyboard");
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::Windows);
    }

    #[test]
    fn test_detect_mac_by_name() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("Apple Magic Keyboard");
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::Mac);
    }

    #[test]
    fn test_detect_mac_by_vendor_id() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("Generic Keyboard").with_vendor_id(0x05ac);
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::Mac);
    }

    #[test]
    fn test_detect_chromebook_by_vendor_id() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("Generic Keyboard").with_vendor_id(0x18d1);
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::Chromebook);
    }

    #[test]
    fn test_detect_ibm_by_vendor_id() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("Generic Keyboard").with_vendor_id(0x17ef);
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::IBM);
    }

    #[test]
    fn test_detect_unknown() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("Generic Unknown Keyboard");
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::Unknown);
    }

    #[test]
    fn test_detect_by_phys_chromebook() {
        let patterns = KeyboardPatterns::new();
        let device = DeviceInfo::new("AT Translated Set 2 keyboard")
            .with_phys("isa0060/serio0/input0");
        // This doesn't match Chromebook path, so should be unknown
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::Unknown);
    }

    #[test]
    fn test_keyboard_type_matches_helper() {
        assert!(keyboard_type_matches(KeyboardType::IBM, "IBM"));
        assert!(keyboard_type_matches(KeyboardType::IBM, "IBM, Mac"));
        assert!(!keyboard_type_matches(KeyboardType::Windows, "IBM, Mac"));
    }

    #[test]
    fn test_detect_simple() {
        let device = DeviceInfo::new("ThinkPad Keyboard");
        assert_eq!(detect_keyboard_type_simple(&device), KeyboardType::IBM);
    }

    #[test]
    fn test_custom_patterns() {
        let patterns = KeyboardPatterns::new()
            .with_ibm_patterns(vec!["custom-ibm"])
            .with_mac_patterns(vec!["custom-mac"]);

        let ibm_device = DeviceInfo::new("My custom-ibm keyboard");
        assert_eq!(detect_keyboard_type(&ibm_device, &patterns), KeyboardType::IBM);

        let mac_device = DeviceInfo::new("My custom-mac keyboard");
        assert_eq!(detect_keyboard_type(&mac_device, &patterns), KeyboardType::Mac);
    }

    #[test]
    fn test_add_vendor_mapping() {
        let mut patterns = KeyboardPatterns::new();
        patterns.add_vendor_mapping(0x9999, KeyboardType::IBM);

        let device = DeviceInfo::new("Unknown").with_vendor_id(0x9999);
        assert_eq!(detect_keyboard_type(&device, &patterns), KeyboardType::IBM);
    }
}

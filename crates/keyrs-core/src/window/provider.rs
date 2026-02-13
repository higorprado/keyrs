// Window Context Provider Trait
//
// This module defines the interface for window context providers,
// which track active window information for conditional keymaps.

use std::fmt;

/// Error type for window context operations
#[derive(Debug, Clone, PartialEq)]
pub enum WindowError {
    /// Not connected to window manager
    NotConnected,

    /// Connection failed
    ConnectionFailed(String),

    /// Query failed
    QueryFailed(String),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::NotConnected => write!(f, "Not connected to window manager"),
            WindowError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            WindowError::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
        }
    }
}

impl std::error::Error for WindowError {}

/// Window information from the active window
///
/// This provides minimal window identification needed for
/// conditional keymaps (e.g., "if in Firefox")
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowInfo {
    /// Window class/app_id (e.g., "firefox", "org.mozilla.firefox")
    pub wm_class: Option<String>,

    /// Window title (e.g., "GitHub - Claude Code")
    pub wm_name: Option<String>,
}

impl WindowInfo {
    /// Create a new empty WindowInfo
    pub fn new() -> Self {
        Self::default()
    }

    /// Create WindowInfo with app_id and title
    pub fn with_details(wm_class: Option<String>, wm_name: Option<String>) -> Self {
        Self { wm_class, wm_name }
    }

    /// Check if the window matches a pattern
    ///
    /// This is used for conditional keymaps like:
    /// - `wm_class == "firefox"` - exact match
    /// - `wm_class =~ "Firefox"` - substring match (regex-like)
    pub fn matches_condition(&self, condition: &WindowCondition) -> bool {
        match condition {
            WindowCondition::WmClassEquals(class) => {
                self.wm_class.as_ref().map_or(false, |c| c == class)
            }
            WindowCondition::WmClassContains(pattern) => {
                self.wm_class.as_ref().map_or(false, |c| {
                    c.to_lowercase().contains(&pattern.to_lowercase())
                })
            }
            WindowCondition::WmNameEquals(name) => {
                self.wm_name.as_ref().map_or(false, |n| n == name)
            }
            WindowCondition::WmNameContains(pattern) => self.wm_name.as_ref().map_or(false, |n| {
                n.to_lowercase().contains(&pattern.to_lowercase())
            }),
        }
    }
}

/// Condition for matching windows
///
/// These correspond to the condition types supported in the configuration:
/// - `wm_class == "value"` - exact match
/// - `wm_class =~ "pattern"` - substring match (case-insensitive)
/// - `wm_name == "value"` - exact match
/// - `wm_name =~ "pattern"` - substring match (case-insensitive)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowCondition {
    /// Exact match on wm_class
    WmClassEquals(String),

    /// Substring match on wm_class (case-insensitive)
    WmClassContains(String),

    /// Exact match on wm_name
    WmNameEquals(String),

    /// Substring match on wm_name (case-insensitive)
    WmNameContains(String),
}

/// Error parsing a window condition string
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionParseError {
    /// Empty condition string
    Empty,
    /// Missing operator (== or =~)
    MissingOperator,
    /// Invalid field name
    InvalidField(String),
    /// Invalid operator
    InvalidOperator(String),
    /// Unquoted value
    UnquotedValue(String),
}

impl fmt::Display for ConditionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConditionParseError::Empty => write!(f, "Empty condition string"),
            ConditionParseError::MissingOperator => write!(f, "Missing operator (== or =~)"),
            ConditionParseError::InvalidField(field) => write!(f, "Invalid field: {}", field),
            ConditionParseError::InvalidOperator(op) => write!(f, "Invalid operator: {}", op),
            ConditionParseError::UnquotedValue(val) => write!(f, "Value must be quoted: {}", val),
        }
    }
}

impl std::error::Error for ConditionParseError {}

impl WindowCondition {
    /// Parse a condition string into a WindowCondition
    ///
    /// Supported formats:
    /// - `wm_class == "value"` - exact match
    /// - `wm_class =~ "pattern"` - substring match
    /// - `wm_name == "value"` - exact match  
    /// - `wm_name =~ "pattern"` - substring match
    ///
    /// # Examples
    /// ```
    /// use keyrs_core::window::{WindowCondition, WindowInfo};
    ///
    /// let condition = WindowCondition::parse("wm_class =~ 'Firefox'").unwrap();
    /// let info = WindowInfo::with_details(Some("org.mozilla.firefox".to_string()), None);
    /// assert!(info.matches_condition(&condition));
    /// ```
    pub fn parse(condition: &str) -> Result<Self, ConditionParseError> {
        let trimmed = condition.trim();
        
        if trimmed.is_empty() {
            return Err(ConditionParseError::Empty);
        }

        // Find the operator (== or =~)
        let (field, op, value) = if let Some(pos) = trimmed.find("==") {
            let field = trimmed[..pos].trim();
            let value = trimmed[pos + 2..].trim();
            (field, "==", value)
        } else if let Some(pos) = trimmed.find("=~") {
            let field = trimmed[..pos].trim();
            let value = trimmed[pos + 2..].trim();
            (field, "=~", value)
        } else {
            return Err(ConditionParseError::MissingOperator);
        };

        // Validate field
        if field != "wm_class" && field != "wm_name" {
            return Err(ConditionParseError::InvalidField(field.to_string()));
        }

        // Strip quotes from value (both single and double quotes)
        let value = if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            value[1..value.len() - 1].to_string()
        } else {
            return Err(ConditionParseError::UnquotedValue(value.to_string()));
        };

        // Create the appropriate condition
        match (field, op) {
            ("wm_class", "==") => Ok(WindowCondition::WmClassEquals(value)),
            ("wm_class", "=~") => Ok(WindowCondition::WmClassContains(value)),
            ("wm_name", "==") => Ok(WindowCondition::WmNameEquals(value)),
            ("wm_name", "=~") => Ok(WindowCondition::WmNameContains(value)),
            (_, _) => Err(ConditionParseError::InvalidOperator(op.to_string())),
        }
    }

    /// Get the field name (wm_class or wm_name)
    pub fn field(&self) -> &'static str {
        match self {
            WindowCondition::WmClassEquals(_) | WindowCondition::WmClassContains(_) => "wm_class",
            WindowCondition::WmNameEquals(_) | WindowCondition::WmNameContains(_) => "wm_name",
        }
    }

    /// Get the pattern/value
    pub fn pattern(&self) -> &str {
        match self {
            WindowCondition::WmClassEquals(s)
            | WindowCondition::WmClassContains(s)
            | WindowCondition::WmNameEquals(s)
            | WindowCondition::WmNameContains(s) => s,
        }
    }

    /// Check if this is an exact match condition
    pub fn is_exact(&self) -> bool {
        matches!(
            self,
            WindowCondition::WmClassEquals(_) | WindowCondition::WmNameEquals(_)
        )
    }

    /// Check if this is a contains/regex match condition
    pub fn is_contains(&self) -> bool {
        matches!(
            self,
            WindowCondition::WmClassContains(_) | WindowCondition::WmNameContains(_)
        )
    }
}

/// Trait for window context providers
///
/// Implementations of this trait provide active window information
/// from different window systems (Wayland, X11, etc.).
pub trait WindowContextProvider: Send + Sync {
    /// Connect to the window manager
    ///
    /// Returns Ok(()) if connection succeeded, Err otherwise.
    /// This may spawn background threads for event handling.
    fn connect(&mut self) -> Result<(), WindowError>;

    /// Disconnect from the window manager
    ///
    /// This should clean up any resources and background threads.
    fn disconnect(&mut self);

    /// Check if connected to the window manager
    fn is_connected(&self) -> bool;

    /// Get the current active window
    ///
    /// Returns WindowInfo with wm_class and wm_name for the
    /// currently focused window.
    fn get_active_window(&self) -> Result<WindowInfo, WindowError>;

    /// Check if window context is available
    ///
    /// This is a convenience method that returns true if connected
    /// and get_active_window() succeeds.
    fn is_available(&self) -> bool {
        self.is_connected() && self.get_active_window().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_info_new() {
        let info = WindowInfo::new();
        assert_eq!(info.wm_class, None);
        assert_eq!(info.wm_name, None);
    }

    #[test]
    fn test_window_info_with_details() {
        let info =
            WindowInfo::with_details(Some("firefox".to_string()), Some("GitHub".to_string()));
        assert_eq!(info.wm_class, Some("firefox".to_string()));
        assert_eq!(info.wm_name, Some("GitHub".to_string()));
    }

    #[test]
    fn test_matches_wm_class_equals() {
        let info = WindowInfo::with_details(Some("org.mozilla.firefox".to_string()), None);

        assert!(info.matches_condition(&WindowCondition::WmClassEquals(
            "org.mozilla.firefox".to_string()
        )));
        assert!(!info.matches_condition(&WindowCondition::WmClassEquals("firefox".to_string())));
    }

    #[test]
    fn test_matches_wm_class_contains() {
        let info = WindowInfo::with_details(Some("org.mozilla.firefox".to_string()), None);

        assert!(info.matches_condition(&WindowCondition::WmClassContains("firefox".to_string())));
        assert!(info.matches_condition(&WindowCondition::WmClassContains("FIREFOX".to_string())));
        assert!(!info.matches_condition(&WindowCondition::WmClassContains("chrome".to_string())));
    }

    #[test]
    fn test_matches_wm_name_equals() {
        let info = WindowInfo::with_details(None, Some("GitHub - Claude Code".to_string()));

        assert!(info.matches_condition(&WindowCondition::WmNameEquals(
            "GitHub - Claude Code".to_string()
        )));
        assert!(!info.matches_condition(&WindowCondition::WmNameEquals("GitHub".to_string())));
    }

    #[test]
    fn test_matches_wm_name_contains() {
        let info = WindowInfo::with_details(None, Some("GitHub - Claude Code".to_string()));

        assert!(info.matches_condition(&WindowCondition::WmNameContains("Claude".to_string())));
        assert!(info.matches_condition(&WindowCondition::WmNameContains("GITHUB".to_string())));
        assert!(!info.matches_condition(&WindowCondition::WmNameContains("VS Code".to_string())));
    }

    #[test]
    fn test_matches_none_values() {
        let info = WindowInfo::new();

        assert!(!info.matches_condition(&WindowCondition::WmClassEquals("firefox".to_string())));
        assert!(!info.matches_condition(&WindowCondition::WmClassContains("firefox".to_string())));
        assert!(!info.matches_condition(&WindowCondition::WmNameEquals("GitHub".to_string())));
        assert!(!info.matches_condition(&WindowCondition::WmNameContains("Claude".to_string())));
    }

    #[test]
    fn test_window_error_display() {
        assert_eq!(
            format!("{}", WindowError::NotConnected),
            "Not connected to window manager"
        );
        assert_eq!(
            format!("{}", WindowError::ConnectionFailed("test".to_string())),
            "Connection failed: test"
        );
        assert_eq!(
            format!("{}", WindowError::QueryFailed("query".to_string())),
            "Query failed: query"
        );
    }

    #[test]
    fn test_window_condition_equality() {
        let c1 = WindowCondition::WmClassEquals("firefox".to_string());
        let c2 = WindowCondition::WmClassEquals("firefox".to_string());
        let c3 = WindowCondition::WmClassContains("firefox".to_string());

        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    // Tests for WindowCondition::parse()
    #[test]
    fn test_parse_wm_class_equals() {
        let condition = WindowCondition::parse("wm_class == \"firefox\"").unwrap();
        assert_eq!(condition, WindowCondition::WmClassEquals("firefox".to_string()));
    }

    #[test]
    fn test_parse_wm_class_equals_single_quotes() {
        let condition = WindowCondition::parse("wm_class == 'firefox'").unwrap();
        assert_eq!(condition, WindowCondition::WmClassEquals("firefox".to_string()));
    }

    #[test]
    fn test_parse_wm_class_contains() {
        let condition = WindowCondition::parse("wm_class =~ \"Firefox\"").unwrap();
        assert_eq!(condition, WindowCondition::WmClassContains("Firefox".to_string()));
    }

    #[test]
    fn test_parse_wm_name_equals() {
        let condition = WindowCondition::parse("wm_name == \"GitHub\"").unwrap();
        assert_eq!(condition, WindowCondition::WmNameEquals("GitHub".to_string()));
    }

    #[test]
    fn test_parse_wm_name_contains() {
        let condition = WindowCondition::parse("wm_name =~ \"Claude\"").unwrap();
        assert_eq!(condition, WindowCondition::WmNameContains("Claude".to_string()));
    }

    #[test]
    fn test_parse_with_whitespace() {
        let condition = WindowCondition::parse("  wm_class  =~  'firefox'  ").unwrap();
        assert_eq!(condition, WindowCondition::WmClassContains("firefox".to_string()));
    }

    #[test]
    fn test_parse_empty_condition() {
        let result = WindowCondition::parse("");
        assert_eq!(result, Err(ConditionParseError::Empty));
    }

    #[test]
    fn test_parse_missing_operator() {
        let result = WindowCondition::parse("wm_class firefox");
        assert_eq!(result, Err(ConditionParseError::MissingOperator));
    }

    #[test]
    fn test_parse_invalid_field() {
        let result = WindowCondition::parse("invalid_field =~ 'firefox'");
        assert_eq!(result, Err(ConditionParseError::InvalidField("invalid_field".to_string())));
    }

    #[test]
    fn test_parse_unquoted_value() {
        let result = WindowCondition::parse("wm_class =~ firefox");
        assert_eq!(result, Err(ConditionParseError::UnquotedValue("firefox".to_string())));
    }

    #[test]
    fn test_parse_partially_quoted_value() {
        let result = WindowCondition::parse("wm_class =~ \"firefox");
        assert_eq!(result, Err(ConditionParseError::UnquotedValue("\"firefox".to_string())));
    }

    #[test]
    fn test_condition_field_method() {
        let c1 = WindowCondition::WmClassEquals("firefox".to_string());
        let c2 = WindowCondition::WmNameContains("github".to_string());

        assert_eq!(c1.field(), "wm_class");
        assert_eq!(c2.field(), "wm_name");
    }

    #[test]
    fn test_condition_pattern_method() {
        let c1 = WindowCondition::WmClassEquals("firefox".to_string());
        let c2 = WindowCondition::WmNameContains("github".to_string());

        assert_eq!(c1.pattern(), "firefox");
        assert_eq!(c2.pattern(), "github");
    }

    #[test]
    fn test_condition_is_exact() {
        let exact = WindowCondition::WmClassEquals("firefox".to_string());
        let contains = WindowCondition::WmClassContains("firefox".to_string());

        assert!(exact.is_exact());
        assert!(!contains.is_exact());
    }

    #[test]
    fn test_condition_is_contains() {
        let exact = WindowCondition::WmClassEquals("firefox".to_string());
        let contains = WindowCondition::WmClassContains("firefox".to_string());

        assert!(!exact.is_contains());
        assert!(contains.is_contains());
    }

    #[test]
    fn test_condition_parse_error_display() {
        assert_eq!(
            format!("{}", ConditionParseError::Empty),
            "Empty condition string"
        );
        assert_eq!(
            format!("{}", ConditionParseError::MissingOperator),
            "Missing operator (== or =~)"
        );
        assert_eq!(
            format!("{}", ConditionParseError::InvalidField("test".to_string())),
            "Invalid field: test"
        );
        assert_eq!(
            format!("{}", ConditionParseError::InvalidOperator("!=".to_string())),
            "Invalid operator: !="
        );
        assert_eq!(
            format!("{}", ConditionParseError::UnquotedValue("val".to_string())),
            "Value must be quoted: val"
        );
    }

    #[test]
    fn test_parse_and_match_integration() {
        // Parse a condition and verify it matches correctly
        let condition = WindowCondition::parse("wm_class =~ 'firefox'").unwrap();
        
        let matching_info = WindowInfo::with_details(
            Some("org.mozilla.firefox".to_string()),
            None
        );
        let non_matching_info = WindowInfo::with_details(
            Some("google-chrome".to_string()),
            None
        );

        assert!(matching_info.matches_condition(&condition));
        assert!(!non_matching_info.matches_condition(&condition));
    }

    #[test]
    fn test_parse_complex_pattern() {
        // Test parsing patterns with spaces and special characters
        let condition = WindowCondition::parse("wm_name =~ 'GitHub - Claude Code'").unwrap();
        assert_eq!(
            condition,
            WindowCondition::WmNameContains("GitHub - Claude Code".to_string())
        );

        let info = WindowInfo::with_details(
            None,
            Some("GitHub - Claude Code".to_string())
        );
        assert!(info.matches_condition(&condition));
    }
}

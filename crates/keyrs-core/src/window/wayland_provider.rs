// Wayland Window Context Provider
//
// This module wraps the existing WaylandClient to implement
// the WindowContextProvider trait.

use super::provider::{WindowContextProvider, WindowError, WindowInfo};
use super::wayland::WaylandClient;

/// Wayland-specific implementation of WindowContextProvider
///
/// This wraps the existing WaylandClient which handles
/// wlroots-based compositors via the foreign-toplevel protocol.
pub struct WaylandContextProvider {
    /// The underlying Wayland client
    client: WaylandClient,
}

impl WaylandContextProvider {
    /// Create a new Wayland context provider
    pub fn new() -> Self {
        Self {
            client: WaylandClient::new(),
        }
    }

    /// Get the underlying Wayland client
    pub fn client(&self) -> &WaylandClient {
        &self.client
    }
}

impl Default for WaylandContextProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowContextProvider for WaylandContextProvider {
    /// Connect to Wayland display
    ///
    /// Spawns a background thread to handle Wayland events.
    fn connect(&mut self) -> Result<(), WindowError> {
        if self.client.connect() {
            Ok(())
        } else {
            Err(WindowError::ConnectionFailed(
                "Failed to connect to Wayland display".to_string(),
            ))
        }
    }

    /// Disconnect from Wayland
    ///
    /// Note: The current WaylandClient doesn't support
    /// explicit disconnection. The background thread will
    /// terminate when the Wayland connection is lost.
    fn disconnect(&mut self) {
        // TODO: Add disconnect support to WaylandClient
        // For now, we rely on drop behavior
    }

    /// Check if connected to Wayland
    fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// Get the current active window
    ///
    /// Returns WindowInfo with wm_class (app_id) and wm_name (title).
    fn get_active_window(&self) -> Result<WindowInfo, WindowError> {
        if !self.is_connected() {
            return Err(WindowError::NotConnected);
        }

        let (app_id, title) = self.client.active_window();

        // Handle error values
        let wm_class = if app_id.starts_with("ERR_") {
            None
        } else {
            Some(app_id)
        };

        let wm_name = if title.starts_with("ERR_") {
            None
        } else {
            Some(title)
        };

        Ok(WindowInfo { wm_class, wm_name })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_provider_new() {
        let provider = WaylandContextProvider::new();
        assert!(!provider.is_connected());
    }

    #[test]
    fn test_wayland_provider_default() {
        let provider = WaylandContextProvider::default();
        assert!(!provider.is_connected());
    }

    #[test]
    fn test_wayland_provider_get_active_window_not_connected() {
        let provider = WaylandContextProvider::new();
        match provider.get_active_window() {
            Err(WindowError::NotConnected) => {
                // Expected
            }
            _ => {
                panic!("Should return NotConnected error");
            }
        }
    }

    #[test]
    fn test_wayland_provider_get_active_window_with_error_values() {
        let mut provider = WaylandContextProvider::new();

        // Simulate error values from WaylandClient
        provider.client.update_active_window(
            "ERR_no_wlr_app_class".to_string(),
            "ERR_no_wlr_wdw_title".to_string(),
        );

        let info = provider.get_active_window().unwrap();
        assert_eq!(info.wm_class, None);
        assert_eq!(info.wm_name, None);
    }

    #[test]
    fn test_wayland_provider_get_active_window_success() {
        let mut provider = WaylandContextProvider::new();

        // Set valid window info
        provider.client.update_active_window(
            "org.mozilla.firefox".to_string(),
            "GitHub - Claude Code".to_string(),
        );

        let info = provider.get_active_window().unwrap();
        assert_eq!(info.wm_class, Some("org.mozilla.firefox".to_string()));
        assert_eq!(info.wm_name, Some("GitHub - Claude Code".to_string()));
    }

    #[test]
    fn test_wayland_provider_connect() {
        // This test requires a running Wayland session
        let mut provider = WaylandContextProvider::new();

        // Try to connect (will fail if no Wayland display)
        let result = provider.connect();

        // If WAYLAND_DISPLAY is not set, connection should fail
        if std::env::var("WAYLAND_DISPLAY").is_err() {
            assert!(result.is_err());
        }

        // After failed connect, should not be connected
        if result.is_err() {
            assert!(!provider.is_connected());
        }
    }

    #[test]
    fn test_wayland_provider_client_access() {
        let provider = WaylandContextProvider::new();
        let _client = provider.client();
        // Just verify we can access the client
    }
}

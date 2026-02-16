//! Wayland window context using wlr-foreign-toplevel-management-unstable-v1
//!
//! This module handles the connection to Wayland compositors and tracks
//! window focus, app_id, and title for active windows on wlroots-based compositors.

use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;

use wayland_backend::rs::client::ObjectId;
use wayland_client::{
    event_created_child,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_display, wl_registry, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1, zwlr_foreign_toplevel_manager_v1,
};

/// Default error values when window info is not available
pub const ERR_NO_APP_CLASS: &str = "ERR_no_wlr_app_class";
pub const ERR_NO_WDW_TITLE: &str = "ERR_no_wlr_wdw_title";

/// Active window context - the current focused window
#[derive(Debug, Clone, Default)]
pub struct ActiveWindow {
    pub app_id: String,
    pub title: String,
}

impl ActiveWindow {
    /// Create a new ActiveWindow with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the active window information
    pub fn update(&mut self, app_id: String, title: String) {
        self.app_id = app_id;
        self.title = title;
    }
}

/// Window information stored for each toplevel handle
#[derive(Debug, Clone)]
struct WindowInfo {
    app_id: String,
    title: String,
    activated: bool,
}

impl WindowInfo {
    fn new() -> Self {
        Self {
            app_id: ERR_NO_APP_CLASS.to_string(),
            title: ERR_NO_WDW_TITLE.to_string(),
            activated: false,
        }
    }
}

/// Wayland client state
///
/// This struct holds all the state for our Wayland client including
/// the connection, event queue, and tracked windows.
struct WaylandState {
    /// Map of toplevel handles to their window info
    windows: HashMap<ObjectId, WindowInfo>,
    /// The currently active/focused window handle
    active_handle: Option<ObjectId>,
    /// Cached active window info for fast access
    active_window: Arc<Mutex<ActiveWindow>>,
}

impl WaylandState {
    fn new(active_window: Arc<Mutex<ActiveWindow>>) -> Self {
        Self {
            windows: HashMap::new(),
            active_handle: None,
            active_window,
        }
    }

    /// Update the cached active window info from the current active handle
    fn update_active_window_cache(&self) {
        if let Some(handle) = &self.active_handle {
            if let Some(info) = self.windows.get(handle) {
                let app_id = if info.app_id.is_empty() {
                    ERR_NO_APP_CLASS
                } else {
                    &info.app_id
                };
                let title = if info.title.is_empty() {
                    ERR_NO_WDW_TITLE
                } else {
                    &info.title
                };

                let mut window = self.active_window.lock().unwrap();
                window.update(app_id.to_string(), title.to_string());
            }
        }
    }
}

// Implement Dispatch for wl_registry
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _globals: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        // Registry events are handled via the globals list
    }
}

// Implement Dispatch for the toplevel manager
impl Dispatch<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _manager: &zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
        event: zwlr_foreign_toplevel_manager_v1::Event,
        _: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                state.windows.insert(toplevel.id(), WindowInfo::new());
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                // The manager is finished, no more toplevels will be sent
            }
            _ => {}
        }
    }

    event_created_child!(WaylandState, zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, [
        0 => (zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ())
    ]);
}

// Implement Dispatch for the toplevel handle
impl Dispatch<zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        handle: &zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        _: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                let handle_id = handle.id();
                if let Some(info) = state.windows.get_mut(&handle_id) {
                    info.title = title;
                    if state.active_handle.as_ref() == Some(&handle_id) {
                        state.update_active_window_cache();
                    }
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                let handle_id = handle.id();
                if let Some(info) = state.windows.get_mut(&handle_id) {
                    info.app_id = app_id;
                    if state.active_handle.as_ref() == Some(&handle_id) {
                        state.update_active_window_cache();
                    }
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::State { state: state_array } => {
                const ACTIVATED_VALUE: u8 = 2;
                let activated = state_array.contains(&ACTIVATED_VALUE);
                let handle_id = handle.id();

                if let Some(info) = state.windows.get_mut(&handle_id) {
                    info.activated = activated;

                    if activated {
                        state.active_handle = Some(handle_id);
                        state.update_active_window_cache();
                    } else if state.active_handle.as_ref() == Some(&handle_id) {
                        state.active_handle = None;
                        let mut window = state.active_window.lock().unwrap();
                        window.update(ERR_NO_APP_CLASS.to_string(), ERR_NO_WDW_TITLE.to_string());
                    }
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::Done => {
                // All pending state changes have been applied
            }
            zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                let handle_id = handle.id();
                if state.windows.remove(&handle_id).is_some() {
                    if state.active_handle.as_ref() == Some(&handle_id) {
                        state.active_handle = None;
                        let mut window = state.active_window.lock().unwrap();
                        window.update(ERR_NO_APP_CLASS.to_string(), ERR_NO_WDW_TITLE.to_string());
                    }
                }
                handle.destroy();
            }
            zwlr_foreign_toplevel_handle_v1::Event::Parent { .. } => {
                // Parent relationship - not needed for our use case
            }
            _ => {}
        }
    }
}

// Empty dispatch implementations for other types
impl Dispatch<wl_surface::WlSurface, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_display::WlDisplay, GlobalListContents> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_display::WlDisplay,
        _event: wl_display::Event,
        _globals: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

/// Wayland client for wlroots compositors
///
/// This client connects to the Wayland display and tracks window focus,
/// app_id, and title for the active window using the wlr-foreign-toplevel
/// management protocol.
pub struct WaylandClient {
    /// Active window information (thread-safe)
    active_window: Arc<Mutex<ActiveWindow>>,
    /// Connection status (thread-safe)
    connected: Arc<Mutex<bool>>,
    /// Event loop thread handle (thread-safe)
    event_thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl WaylandClient {
    fn parse_wayland_display_suffix(name: &str) -> Option<u32> {
        let suffix = name.strip_prefix("wayland-")?;
        if suffix.is_empty() || !suffix.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        suffix.parse::<u32>().ok()
    }

    fn discover_wayland_displays() -> Vec<String> {
        let runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => return Vec::new(),
        };

        let mut displays: Vec<(u32, String)> = Vec::new();
        if let Ok(entries) = fs::read_dir(runtime_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Some(name) = name.to_str() {
                    if let Some(order) = Self::parse_wayland_display_suffix(name) {
                        displays.push((order, name.to_string()));
                    }
                }
            }
        }

        displays.sort_by(|a, b| b.0.cmp(&a.0));
        displays.into_iter().map(|(_, name)| name).collect()
    }

    /// Create a new Wayland client
    pub fn new() -> Self {
        Self {
            active_window: Arc::new(Mutex::new(ActiveWindow::new())),
            connected: Arc::new(Mutex::new(false)),
            event_thread: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to the Wayland display
    ///
    /// Returns true if connection was successful, false otherwise.
    /// This spawns a background thread to handle Wayland events.
    pub fn connect(&self) -> bool {
        // Try WAYLAND_DISPLAY first when set; otherwise probe discovered
        // wayland-* sockets under XDG_RUNTIME_DIR.
        let mut candidates = Vec::new();
        for display in Self::discover_wayland_displays() {
            if !candidates.iter().any(|existing| existing == &display) {
                candidates.push(display);
            }
        }
        if let Ok(display) = std::env::var("WAYLAND_DISPLAY") {
            if !display.trim().is_empty()
                && !candidates.iter().any(|existing| existing == &display)
            {
                candidates.push(display);
            }
        }
        if candidates.is_empty() {
            return false;
        }

        // Try each candidate display until one works.
        let mut connection = None;
        for display in candidates {
            std::env::set_var("WAYLAND_DISPLAY", &display);
            if let Ok(conn) = Connection::connect_to_env() {
                connection = Some(conn);
                break;
            }
        }
        let connection = match connection {
            Some(conn) => conn,
            None => return false,
        };

        // Initialize the registry queue with WaylandState
        let (globals, mut event_queue) = match registry_queue_init::<WaylandState>(&connection) {
            Ok(g) => g,
            Err(_) => return false,
        };
        let qhandle = event_queue.handle();

        // Bind to the toplevel manager
        let toplevel_manager = match globals
            .bind::<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, _, _>(
            &qhandle,
            3..=3,
            (),
        ) {
            Ok(m) => m,
            Err(_) => return false,
        };
        let _ = toplevel_manager;

        // Create the state
        let mut state = WaylandState::new(self.active_window.clone());

        // Set up the event processing in a background thread
        let connected_flag = self.connected.clone();

        let handle = thread::spawn(move || {
            *connected_flag.lock().unwrap() = true;

            // Do a roundtrip to get initial toplevels
            let _ = event_queue.roundtrip(&mut state);

            // Keep the thread alive to process events
            loop {
                match event_queue.blocking_dispatch(&mut state) {
                    Ok(_) => {}
                    Err(_) => {
                        *connected_flag.lock().unwrap() = false;
                        break;
                    }
                }
            }
        });

        *self.event_thread.lock().unwrap() = Some(handle);

        true
    }

    /// Get the current active window information
    pub fn active_window(&self) -> (String, String) {
        let window = self.active_window.lock().unwrap();
        (window.app_id.clone(), window.title.clone())
    }

    /// Update the active window (for internal use/testing)
    pub fn update_active_window(&self, app_id: String, title: String) {
        let mut window = self.active_window.lock().unwrap();
        window.update(app_id, title);
    }

    /// Check if connected to Wayland
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    #[cfg(test)]
    pub(crate) fn set_connected_for_test(&self, connected: bool) {
        *self.connected.lock().unwrap() = connected;
    }
}

impl Default for WaylandClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_window_new() {
        let window = ActiveWindow::new();
        assert_eq!(window.app_id, "");
        assert_eq!(window.title, "");
    }

    #[test]
    fn test_active_window_update() {
        let mut window = ActiveWindow::new();
        window.update("firefox".to_string(), "Test Title".to_string());
        assert_eq!(window.app_id, "firefox");
        assert_eq!(window.title, "Test Title");
    }

    #[test]
    fn test_window_info_new() {
        let info = WindowInfo::new();
        assert_eq!(info.app_id, ERR_NO_APP_CLASS);
        assert_eq!(info.title, ERR_NO_WDW_TITLE);
        assert!(!info.activated);
    }

    #[test]
    fn test_wayland_client_new() {
        let client = WaylandClient::new();
        assert!(!client.is_connected());
    }

    #[test]
    fn test_error_constants() {
        assert_eq!(ERR_NO_APP_CLASS, "ERR_no_wlr_app_class");
        assert_eq!(ERR_NO_WDW_TITLE, "ERR_no_wlr_wdw_title");
    }

    #[test]
    fn test_wayland_client_update_active_window() {
        let client = WaylandClient::new();
        client.update_active_window("test_app".to_string(), "Test Window".to_string());

        let (app_id, title) = client.active_window();
        assert_eq!(app_id, "test_app");
        assert_eq!(title, "Test Window");
    }

    #[test]
    fn test_discover_wayland_displays() {
        let tmp = std::env::temp_dir().join(format!(
            "keyrs-wayland-displays-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("wayland-1"), b"").unwrap();
        fs::write(tmp.join("wayland-0"), b"").unwrap();
        fs::write(tmp.join("wayland-1.lock"), b"").unwrap();
        fs::write(tmp.join("wayland-1-awww-daemon..sock"), b"").unwrap();
        fs::write(tmp.join("wayland-abc"), b"").unwrap();
        fs::write(tmp.join("not-wayland"), b"").unwrap();

        let prev = std::env::var("XDG_RUNTIME_DIR").ok();
        std::env::set_var("XDG_RUNTIME_DIR", &tmp);
        let displays = WaylandClient::discover_wayland_displays();
        match prev {
            Some(v) => std::env::set_var("XDG_RUNTIME_DIR", v),
            None => std::env::remove_var("XDG_RUNTIME_DIR"),
        }

        assert_eq!(displays, vec!["wayland-1".to_string(), "wayland-0".to_string()]);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_parse_wayland_display_suffix() {
        assert_eq!(
            WaylandClient::parse_wayland_display_suffix("wayland-0"),
            Some(0)
        );
        assert_eq!(
            WaylandClient::parse_wayland_display_suffix("wayland-12"),
            Some(12)
        );
        assert_eq!(
            WaylandClient::parse_wayland_display_suffix("wayland-1.lock"),
            None
        );
        assert_eq!(
            WaylandClient::parse_wayland_display_suffix("wayland-1-awww-daemon..sock"),
            None
        );
        assert_eq!(
            WaylandClient::parse_wayland_display_suffix("not-wayland-1"),
            None
        );
    }
}

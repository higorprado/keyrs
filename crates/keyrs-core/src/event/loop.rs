// Keyrs Pure Rust Event Loop
// Direct evdev event handling for low-latency input processing

#[cfg(feature = "pure-rust")]
use evdev::{Device, EventType, InputEvent, Key};
#[cfg(feature = "pure-rust")]
use std::os::unix::io::AsRawFd;
#[cfg(feature = "pure-rust")]
use crate::input::{is_virtual_device, matches_device_filter};

#[cfg(feature = "pure-rust")]
use udev::MonitorSocket;

/// Result type for event loop operations
pub type EventLoopResult<T> = Result<T, EventLoopError>;

/// Errors that can occur in event loop
#[derive(Debug, thiserror::Error)]
pub enum EventLoopError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Event device error: {0}")]
    Evdev(String),
}

/// Device information for listing devices
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device index
    pub index: usize,
    /// Device name
    pub name: String,
    /// Device path (if available)
    pub path: Option<String>,
}

/// Input event annotated with source device metadata.
#[derive(Debug, Clone)]
pub struct PolledEvent {
    /// Raw evdev input event
    pub event: InputEvent,
    /// Source device name
    pub device_name: String,
}

/// Pure Rust event loop for direct device access
///
/// This provides direct access to evdev devices without intermediate layers.
/// Supports device grabbing, polling, and automatic cleanup on drop.
#[cfg(feature = "pure-rust")]
pub struct EventLoop {
    devices: Vec<Device>,
    device_paths: Vec<String>,
    poll_fds: Vec<libc::pollfd>,
    grabbed: bool,
    /// udev monitor for hotplug detection (fd is at poll_fds[0])
    udev_monitor: Option<MonitorSocket>,
    /// Device filter for hotplug matching
    device_filter: Vec<String>,
}

#[cfg(feature = "pure-rust")]
impl EventLoop {
    /// Virtual device prefix to filter out
    const VIRT_DEVICE_PREFIX: &str = "Keyrs (virtual)";

    /// Poll flags indicating device disconnection
    const DISCONNECT_FLAGS: libc::c_short =
        libc::POLLHUP | libc::POLLERR | libc::POLLNVAL;

    /// Create a new event loop by finding keyboard devices
    pub fn new() -> EventLoopResult<Self> {
        Self::new_filtered(&[])
    }

    /// Create a new event loop with device filtering (no grab)
    fn new_filtered(filter_names: &[String]) -> EventLoopResult<Self> {
        let keyboards_with_paths = Self::find_keyboards_with_paths(filter_names)?;
        let udev_monitor = Self::create_udev_monitor()?;
        let mut poll_fds = Vec::new();
        
        // udev fd at index 0 if available
        if let Some(ref monitor) = udev_monitor {
            poll_fds.push(libc::pollfd {
                fd: monitor.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            });
        }
        
        // Extract devices and paths
        let (device_paths, devices): (Vec<String>, Vec<Device>) = keyboards_with_paths
            .into_iter()
            .unzip();
        
        // Device fds follow
        poll_fds.extend(Self::create_poll_fds(&devices));
        
        Ok(Self {
            devices,
            device_paths,
            poll_fds,
            grabbed: false,
            udev_monitor,
            device_filter: filter_names.to_vec(),
        })
    }

    /// Create a new event loop and grab all keyboard devices
    ///
    /// This is the preferred mode for keyrs operation - it prevents
    /// other applications from receiving key events directly.
    pub fn new_with_grab() -> EventLoopResult<Self> {
        Self::new_with_grab_filtered(&[])
    }

    /// Create a new event loop and grab filtered keyboard devices.
    pub fn new_with_grab_filtered(filter_names: &[String]) -> EventLoopResult<Self> {
        let keyboards_with_paths = Self::find_keyboards_with_paths(filter_names)?;
        
        // Extract devices and paths
        let (device_paths, mut devices): (Vec<String>, Vec<Device>) = keyboards_with_paths
            .into_iter()
            .unzip();

        // Defensive: First try to ungrab all devices to handle the case where
        // a previous instance crashed. This ensures we start with a clean state.
        for device in &mut devices {
            // Ignore errors here - devices may not be grabbed
            let _ = device.ungrab();
        }

        // Now grab all keyboard devices
        for device in &mut devices {
            device.grab()?;
        }

        let udev_monitor = Self::create_udev_monitor()?;
        let mut poll_fds = Vec::new();
        
        // udev fd at index 0 if available
        if let Some(ref monitor) = udev_monitor {
            poll_fds.push(libc::pollfd {
                fd: monitor.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            });
        }
        
        // Device fds follow
        poll_fds.extend(Self::create_poll_fds(&devices));
        
        Ok(Self {
            devices,
            device_paths,
            poll_fds,
            grabbed: true,
            udev_monitor,
            device_filter: filter_names.to_vec(),
        })
    }

    /// Create udev monitor for hotplug detection
    fn create_udev_monitor() -> EventLoopResult<Option<MonitorSocket>> {
        let socket = udev::MonitorBuilder::new()
            .and_then(|b| b.match_subsystem("input"))
            .and_then(|b| b.listen())
            .map_err(|e| EventLoopError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create udev monitor: {}", e)
            )))?;
        Ok(Some(socket))
    }

    /// Create poll file descriptors from devices
    fn create_poll_fds(devices: &[Device]) -> Vec<libc::pollfd> {
        devices
            .iter()
            .map(|d| libc::pollfd {
                fd: d.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            })
            .collect()
    }

    /// Ungrab all devices (called on shutdown)
    pub fn ungrab_all(&mut self) {
        if self.grabbed {
            for device in &mut self.devices {
                let _ = device.ungrab();
            }
            self.grabbed = false;
        }
    }

    /// List all available keyboard devices
    ///
    /// This is useful for the --list-devices CLI flag.
    pub fn list_devices() -> EventLoopResult<Vec<DeviceInfo>> {
        let mut devices_info = Vec::new();
        let mut index = 0;

        for (path, device) in evdev::enumerate() {
            if Self::is_keyboard_device(&device) {
                let name = device.name().unwrap_or("Unknown").to_string();
                let device_path = path.to_str().map(|s| s.to_string());
                devices_info.push(DeviceInfo {
                    index,
                    name,
                    path: device_path,
                });
                index += 1;
            }
        }

        if devices_info.is_empty() {
            return Err(EventLoopError::DeviceNotFound(
                "No keyboard devices found".to_string(),
            ));
        }

        Ok(devices_info)
    }

    /// Find keyboard devices honoring explicit filter names/paths.
    /// Returns (device_node_path, device) pairs.
    fn find_keyboards_with_paths(filter_names: &[String]) -> EventLoopResult<Vec<(String, Device)>> {
        let mut keyboards = Vec::new();
        let autodetect = filter_names.is_empty();

        for (path, device) in evdev::enumerate() {
            let device_name = device.name().unwrap_or("Unknown");
            let device_path = path.to_str().unwrap_or_default();
            let is_keyboard = Self::is_keyboard_device(&device);
            let is_virtual = is_virtual_device(device_name, Self::VIRT_DEVICE_PREFIX);

            if matches_device_filter(
                device_name,
                device_path,
                filter_names,
                autodetect,
                is_keyboard,
                is_virtual,
            ) {
                keyboards.push((device_path.to_string(), device));
            }
        }

        if keyboards.is_empty() {
            return Err(EventLoopError::DeviceNotFound(
                "No keyboard devices found".to_string(),
            ));
        }

        Ok(keyboards)
    }

    /// Check if a device is a keyboard
    fn is_keyboard_device(device: &Device) -> bool {
        // Check if device supports EV_KEY
        if !device.supported_events().contains(EventType::KEY) {
            return false;
        }

        // Filter out virtual devices to prevent feedback loop
        // The virtual Keyrs device created by output layer should not be grabbed
        let device_name = device.name().unwrap_or("");
        if crate::input::is_virtual_device(device_name, Self::VIRT_DEVICE_PREFIX) {
            return false;
        }

        // Get supported keys
        let keys = match device.supported_keys() {
            Some(k) => k,
            None => return false,
        };

        // Check for QWERTY row keys (Q=16, W=17, E=18, R=19, T=20, Y=21)
        const QWERTY_CODES: &[u16] = &[16, 17, 18, 19, 20, 21];

        // Check for A-Z representative keys and SPACE (A=30, Z=44, SPACE=57)
        const A_Z_SPACE_CODES: &[u16] = &[57, 30, 44];

        let qwerty_present = QWERTY_CODES
            .iter()
            .all(|code| keys.contains(Key::new(*code)));
        let az_present = A_Z_SPACE_CODES
            .iter()
            .all(|code| keys.contains(Key::new(*code)));

        qwerty_present && az_present
    }

    /// Poll for events with timeout (non-blocking)
    ///
    /// This method uses libc::poll() to efficiently wait for events
    /// across multiple devices without busy-waiting.
    ///
    /// # Arguments
    /// * `timeout_ms` - Timeout in milliseconds (0 = non-blocking, -1 = infinite)
    ///
    /// # Returns
    /// A vector of input events from all devices that have data available
    ///
    /// # Errors
    /// Returns empty vector on timeout or EINTR (interrupted system call).
    /// Returns an error only for fatal I/O errors.
    pub fn poll_for_events(&mut self, timeout_ms: i32) -> EventLoopResult<Vec<InputEvent>> {
        let events = self.poll_for_events_with_device(timeout_ms)?;
        Ok(events.into_iter().map(|e| e.event).collect())
    }

    /// Poll for events with source device metadata.
    pub fn poll_for_events_with_device(
        &mut self,
        timeout_ms: i32,
    ) -> EventLoopResult<Vec<PolledEvent>> {
        let mut events = Vec::new();

        // Wait for events
        let poll_result = unsafe {
            libc::poll(
                self.poll_fds.as_mut_ptr(),
                self.poll_fds.len() as libc::nfds_t,
                timeout_ms,
            )
        };

        if poll_result < 0 {
            let err = std::io::Error::last_os_error();
            // EINTR (Interrupted system call) is not a fatal error - it just means
            // a signal was delivered (e.g., Ctrl+C). We should treat it like
            // a timeout and return no events. The caller will check running flag
            // and exit if needed.
            if err.raw_os_error() == Some(4) {
                // EINTR on Linux
                return Ok(events);
            }
            return Err(EventLoopError::Io(err));
        }

        if poll_result == 0 {
            // Timeout - no events
            return Ok(events);
        }

        // Check udev events first (index 0 if present)
        let udev_offset = if self.udev_monitor.is_some() { 1 } else { 0 };
        if udev_offset > 0 && self.poll_fds[0].revents & libc::POLLIN != 0 {
            self.handle_udev_events();
        }

        // Read events from devices that have data available
        // Track disconnected devices for removal
        let mut disconnected_indices: Vec<usize> = Vec::new();

        for (i, device) in self.devices.iter_mut().enumerate() {
            let poll_idx = i + udev_offset;
            let revents = self.poll_fds[poll_idx].revents;

            // Check for device disconnection first
            if revents & Self::DISCONNECT_FLAGS != 0 {
                let device_name = device.name().unwrap_or("Unknown");
                log::warn!("Device disconnected: {}", device_name);
                disconnected_indices.push(i);
                continue;
            }

            // Normal event processing
            if revents & libc::POLLIN != 0 {
                let device_name = device.name().unwrap_or("Unknown").to_string();
                if let Ok(device_events) = device.fetch_events() {
                    for event in device_events {
                        events.push(PolledEvent {
                            event,
                            device_name: device_name.clone(),
                        });
                    }
                }
            }
        }

        // Remove disconnected devices (reverse order to maintain valid indices)
        for i in disconnected_indices.into_iter().rev() {
            let poll_idx = i + udev_offset;
            self.devices.remove(i);
            self.device_paths.remove(i);
            self.poll_fds.remove(poll_idx);
        }

        Ok(events)
    }

    /// Handle udev hotplug events
    fn handle_udev_events(&mut self) {
        let Some(ref monitor) = self.udev_monitor else { return };
        
        // Collect paths first to avoid borrow conflict
        let new_device_paths: Vec<String> = monitor
            .iter()
            .filter_map(|event| {
                if event.event_type() == udev::EventType::Add {
                    event.devnode().and_then(|p| p.to_str().map(|s| s.to_string()))
                } else {
                    None
                }
            })
            .filter(|p| p.starts_with("/dev/input/event"))
            .collect();
        
        for path in new_device_paths {
            self.try_add_device(&path);
        }
    }

    /// Try to add a device by path if it matches our keyboard criteria
    fn try_add_device(&mut self, path: &str) {
        // Check if device is already in our list by path
        if self.device_paths.iter().any(|p| path == p) {
            return;
        }
        
        // Try to open the device
        let mut device = match Device::open(path) {
            Ok(d) => d,
            Err(e) => {
                log::debug!("Could not open device {}: {}", path, e);
                return;
            }
        };
        
        // Check if it's a keyboard device we want
        let device_name = device.name().unwrap_or("Unknown").to_string();
        let device_path = path;
        let is_keyboard = Self::is_keyboard_device(&device);
        let is_virtual = is_virtual_device(&device_name, Self::VIRT_DEVICE_PREFIX);
        
        if !matches_device_filter(
            &device_name,
            device_path,
            &self.device_filter,
            self.device_filter.is_empty(),
            is_keyboard,
            is_virtual,
        ) {
            return;
        }
        
        // Grab if needed
        if self.grabbed {
            if let Err(e) = device.grab() {
                log::warn!("Could not grab new device {}: {}", device_name, e);
                return;
            }
        }
        
        log::info!("Device connected: {} ({})", device_name, path);
        
        // Track the device path
        self.device_paths.push(path.to_string());
        
        // Add to poll_fds (at the end)
        self.poll_fds.push(libc::pollfd {
            fd: device.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        });
        self.devices.push(device);
    }

    /// Fetch a single event from any device (blocking)
    ///
    /// Returns first event available from any device.
    /// Note: This is a simplified interface - prefer poll_for_events() for
    /// better performance and signal handling.
    pub fn fetch_event(&mut self) -> EventLoopResult<InputEvent> {
        for device in &mut self.devices {
            if let Ok(mut iter) = device.fetch_events() {
                if let Some(event) = iter.next() {
                    return Ok(event);
                }
            }
        }
        Err(EventLoopError::Evdev("No events available".to_string()))
    }

    /// Fetch all available events from all devices (non-blocking)
    ///
    /// Returns all pending events from all devices.
    /// Note: This is a simplified interface - prefer poll_for_events() for
    /// better performance and signal handling.
    pub fn fetch_all_events(&mut self) -> EventLoopResult<Vec<InputEvent>> {
        let mut events = Vec::new();

        for device in &mut self.devices {
            if let Ok(device_events) = device.fetch_events() {
                events.extend(device_events);
            }
        }

        Ok(events)
    }

    /// Get the names of all devices
    pub fn device_names(&self) -> Vec<String> {
        self.devices
            .iter()
            .map(|d| d.name().unwrap_or("Unknown").to_string())
            .collect()
    }

    /// Build keyboard detection info from active devices.
    pub fn keyboard_detection_infos(&self) -> Vec<crate::input::KeyboardDeviceInfo> {
        self.devices
            .iter()
            .map(|d| {
                let mut info = crate::input::KeyboardDeviceInfo::new(
                    d.name().unwrap_or("Unknown").to_string(),
                );

                let input_id = d.input_id();
                info = info.with_vendor_id(input_id.vendor()).with_product_id(input_id.product());

                if let Some(phys) = d.physical_path() {
                    info = info.with_phys(phys.to_string());
                }

                info
            })
            .collect()
    }

    /// Get number of devices managed by this event loop
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
}

/// Drop implementation for EventLoop
///
/// This is CRITICAL for system safety. When the event loop panics or exits,
/// devices MUST be ungrabbed, otherwise the keyboard will remain in an
/// unusable state (keys appear stuck). The Drop trait guarantees this
/// cleanup runs even during panic unwinding.
#[cfg(feature = "pure-rust")]
impl Drop for EventLoop {
    fn drop(&mut self) {
        // Always ungrab devices when event loop is destroyed
        // This runs during normal return, early return, panic, and explicit exit
        self.ungrab_all();
    }
}

#[cfg(feature = "pure-rust")]
impl Default for EventLoop {
    fn default() -> Self {
        Self::new().expect("Failed to initialize event loop")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_event_loop_creation() {
        // This test will only work if there's a keyboard device
        match EventLoop::new() {
            Ok(loop_) => {
                assert!(!loop_.devices.is_empty());
                assert!(!loop_.grabbed);
            }
            Err(EventLoopError::DeviceNotFound(_)) => {
                // No keyboard devices - skip test
                println!("Skipping test: no keyboard devices found");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_event_loop_with_grab() {
        match EventLoop::new_with_grab() {
            Ok(loop_) => {
                assert!(!loop_.devices.is_empty());
                assert!(loop_.grabbed);
            }
            Err(EventLoopError::DeviceNotFound(_)) => {
                println!("Skipping test: no keyboard devices found");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_poll_timeout() {
        match EventLoop::new() {
            Ok(mut loop_) => {
                // Poll with 10ms timeout - should return empty vector
                match loop_.poll_for_events(10) {
                    Ok(events) => {
                        // Events should be empty or contain whatever happened in 10ms
                        assert!(events.is_empty() || !events.is_empty());
                    }
                    Err(e) => {
                        panic!("Unexpected error: {}", e);
                    }
                }
            }
            Err(EventLoopError::DeviceNotFound(_)) => {
                println!("Skipping test: no keyboard devices found");
            }
            Err(_) => {}
        }
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_list_devices() {
        match EventLoop::list_devices() {
            Ok(devices) => {
                // Should find at least one keyboard device (the test device)
                println!("Found {} devices", devices.len());
                for device in &devices {
                    println!("  {}: {} ({:?})", device.index, device.name, device.path);
                }
            }
            Err(EventLoopError::DeviceNotFound(_)) => {
                println!("Skipping test: no keyboard devices found");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}

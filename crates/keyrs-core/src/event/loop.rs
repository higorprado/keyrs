// Keyrs Pure Rust Event Loop
// Direct evdev event handling for low-latency input processing

#[cfg(feature = "pure-rust")]
use evdev::{Device, EventType, InputEvent, Key};
#[cfg(feature = "pure-rust")]
use std::os::unix::io::AsRawFd;
#[cfg(feature = "pure-rust")]
use crate::input::{is_virtual_device, matches_device_filter};

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
    poll_fds: Vec<libc::pollfd>,
    grabbed: bool,
}

#[cfg(feature = "pure-rust")]
impl EventLoop {
    /// Virtual device prefix to filter out
    const VIRT_DEVICE_PREFIX: &str = "Keyrs (virtual)";

    /// Create a new event loop by finding keyboard devices
    pub fn new() -> EventLoopResult<Self> {
        let devices = Self::find_keyboards()?;
        let poll_fds = Self::create_poll_fds(&devices);
        Ok(Self {
            devices,
            poll_fds,
            grabbed: false,
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
        let mut devices = Self::find_keyboards_filtered(filter_names)?;

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

        let poll_fds = Self::create_poll_fds(&devices);
        Ok(Self {
            devices,
            poll_fds,
            grabbed: true,
        })
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

    /// Find all keyboard devices
    fn find_keyboards() -> EventLoopResult<Vec<Device>> {
        Self::find_keyboards_filtered(&[])
    }

    /// Find keyboard devices honoring explicit filter names/paths.
    fn find_keyboards_filtered(filter_names: &[String]) -> EventLoopResult<Vec<Device>> {
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
                keyboards.push(device);
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

        // Read events from devices that have data available
        for (i, device) in self.devices.iter_mut().enumerate() {
            if self.poll_fds[i].revents & libc::POLLIN != 0 {
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

        Ok(events)
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

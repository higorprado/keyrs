// Keyrs Hybrid Event Loop
// Rust event I/O with Python transform logic
//
// This module implements a hybrid architecture where:
// - Rust handles low-level event I/O (reading from evdev devices)
// - Python handles transformation logic (via PyO3 bridge)
// - Rust handles output (writing to uinput device)
//
// The goal is to leverage Rust's performance for I/O while maintaining
// Python's flexibility for configuration and transformation logic.

#[cfg(feature = "python-runtime")]
use evdev::{Device, EventType, InputEvent};
#[cfg(feature = "python-runtime")]
use std::os::unix::io::AsRawFd;
#[cfg(feature = "python-runtime")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "python-runtime")]
use std::sync::Arc;
#[cfg(feature = "python-runtime")]
use std::time::{Duration, UNIX_EPOCH};

#[cfg(feature = "python-runtime")]
use pyo3::{PyObject, Python};

use crate::input::is_virtual_device;
use crate::{Action, Combo, ComboHint, Key};

/// Result type for hybrid event loop operations
pub type HybridResult<T> = Result<T, HybridError>;

/// Errors that can occur in the hybrid event loop
#[derive(Debug, thiserror::Error)]
pub enum HybridError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Event device error: {0}")]
    Evdev(String),

    #[error("Python error: {0}")]
    Python(String),

    #[error("Output error: {0}")]
    Output(String),
}

/// Raw input event (from evdev)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawInputEvent {
    /// Event timestamp (seconds)
    pub time_sec: u64,

    /// Event timestamp (microseconds)
    pub time_usec: u64,

    /// Event type (EV_KEY = 0x01)
    pub event_type: u16,

    /// Key code
    pub key_code: u16,

    /// Event value (0=Release, 1=Press, 2=Repeat)
    pub value: u32,
}

impl RawInputEvent {
    /// Check if this is a key event
    pub fn is_key_event(&self) -> bool {
        self.event_type == 0x01 // EV_KEY
    }

    /// Get the action for this event
    pub fn action(&self) -> Action {
        match self.value {
            0 => Action::Release,
            1 => Action::Press,
            2 => Action::Repeat,
            _ => Action::Release,
        }
    }

    /// Get the key for this event
    pub fn key(&self) -> Key {
        Key::from(self.key_code)
    }
}

impl From<InputEvent> for RawInputEvent {
    fn from(event: InputEvent) -> Self {
        // Convert SystemTime to seconds and microseconds since UNIX epoch
        let duration = event
            .timestamp()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);

        Self {
            time_sec: duration.as_secs(),
            time_usec: duration.subsec_micros() as u64,
            event_type: event.event_type().0,
            key_code: event.code(),
            value: event.value() as u32,
        }
    }
}

/// Result of transforming a key event
#[derive(Debug, Clone, PartialEq)]
pub enum TransformResult {
    /// Passthrough - send the key as-is
    Passthrough(Key),

    /// Remapped to a different key
    Remapped(Key),

    /// Combo matched with a combo output (multi-key)
    Combo(Combo),

    /// Special hint (Bind, EscapeNext, etc.)
    Hint(ComboHint),

    /// Suppressed - don't send anything
    Suppress,

    /// Suspend mode activated
    Suspend,
}

/// Event reader for reading from evdev devices
#[cfg(feature = "python-runtime")]
pub struct EventReader {
    /// evdev devices
    devices: Vec<Device>,

    /// Device file descriptors for polling
    poll_fds: Vec<libc::pollfd>,
}

#[cfg(feature = "python-runtime")]
impl EventReader {
    /// Create a new event reader by finding keyboard devices
    pub fn new() -> HybridResult<Self> {
        let devices = Self::find_keyboards()?;
        let poll_fds = devices
            .iter()
            .map(|d| libc::pollfd {
                fd: d.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            })
            .collect();

        Ok(Self { devices, poll_fds })
    }

    /// Create a new event reader and grab all keyboard devices
    pub fn new_with_grab() -> HybridResult<Self> {
        let mut devices = Self::find_keyboards()?;

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

        let poll_fds = devices
            .iter()
            .map(|d| libc::pollfd {
                fd: d.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            })
            .collect();

        Ok(Self { devices, poll_fds })
    }

    /// Ungrab all devices (called on shutdown)
    pub fn ungrab_all(&mut self) {
        for device in &mut self.devices {
            let _ = device.ungrab();
        }
    }

    /// Find all keyboard devices
    fn find_keyboards() -> HybridResult<Vec<Device>> {
        let mut keyboards = Vec::new();

        for (_path, device) in evdev::enumerate() {
            if Self::is_keyboard_device(&device) {
                keyboards.push(device);
            }
        }

        if keyboards.is_empty() {
            return Err(HybridError::DeviceNotFound(
                "No keyboard devices found".to_string(),
            ));
        }

        Ok(keyboards)
    }

    /// Virtual device prefix to filter out
    const VIRT_DEVICE_PREFIX: &str = "Keyrs (virtual)";

    /// Check if a device is a keyboard
    fn is_keyboard_device(device: &Device) -> bool {
        // Check if device supports EV_KEY
        if !device.supported_events().contains(EventType::KEY) {
            return false;
        }

        // Filter out virtual devices to prevent feedback loop
        // The virtual Keyrs device created by the output layer should not be grabbed
        let device_name = device.name().unwrap_or("");
        if is_virtual_device(device_name, Self::VIRT_DEVICE_PREFIX) {
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
            .all(|code| keys.contains(evdev::Key::new(*code)));
        let az_present = A_Z_SPACE_CODES
            .iter()
            .all(|code| keys.contains(evdev::Key::new(*code)));

        qwerty_present && az_present
    }

    /// Poll for events with timeout (non-blocking)
    pub fn poll_for_events(&mut self, timeout_ms: i32) -> HybridResult<Vec<RawInputEvent>> {
        let mut events = Vec::new();

        // Wait for events (without GIL - this is pure Rust)
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
            // a signal was delivered. We should treat it like a timeout and return
            // no events. The caller will check the running flag and exit if needed.
            if err.raw_os_error() == Some(4) {
                // EINTR on Linux
                return Ok(events);
            }
            return Err(HybridError::Io(err));
        }

        if poll_result == 0 {
            // Timeout - no events
            return Ok(events);
        }

        // Read events from devices that have data available
        for (i, device) in self.devices.iter_mut().enumerate() {
            if self.poll_fds[i].revents & libc::POLLIN != 0 {
                if let Ok(device_events) = device.fetch_events() {
                    for event in device_events {
                        events.push(RawInputEvent::from(event));
                    }
                }
            }
        }

        Ok(events)
    }

    /// Get the number of devices managed by this reader
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
}

// PyO3 Bridge for Python Integration
// These types and functions are only available when the python-runtime feature is enabled

/// PyO3 wrapper for the hybrid event loop
///
/// This is exposed to Python and provides the interface for running
/// the Rust event loop with Python transform logic.
#[cfg(feature = "python-runtime")]
pub struct PyHybridEventLoop {
    /// Event reader for reading from evdev devices
    event_reader: EventReader,

    /// Whether the event loop is running
    running: Arc<AtomicBool>,
}

#[cfg(feature = "python-runtime")]
impl PyHybridEventLoop {
    /// Create a new hybrid event loop
    pub fn new() -> HybridResult<Self> {
        let event_reader = EventReader::new()?;

        Ok(Self {
            event_reader,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Create a new hybrid event loop with device grabbing
    ///
    /// This is the preferred mode for hybrid operation - Rust handles
    /// both reading and grabbing devices, while Python handles transformation.
    pub fn new_with_grab() -> HybridResult<Self> {
        let event_reader = EventReader::new_with_grab()?;

        Ok(Self {
            event_reader,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Ungrab all devices (for cleanup)
    pub fn ungrab_all(&mut self) {
        self.event_reader.ungrab_all();
    }

    /// Get the number of devices
    pub fn device_count(&self) -> usize {
        self.event_reader.device_count()
    }

    /// Stop the event loop
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check if the loop is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Run the event loop with a Python callback.
    ///
    /// This method:
    /// 1. Sets the running flag
    /// 2. Installs a panic handler for logging (Drop trait handles cleanup)
    /// 3. Loops until stopped or interrupted
    /// 4. For each iteration:
    ///    a. Releases the GIL (allow_threads) for efficient polling
    ///    b. Polls for events (pure Rust, no GIL)
    ///    c. Acquires the GIL
    ///    d. Calls the Python callback for each key event
    ///
    /// # Safety
    /// This function is safe because the Drop trait on PyHybridEventLoop
    /// guarantees that devices are ungrabbed even during panic unwinding.
    ///
    /// # Arguments
    /// * `py` - The Python token (GIL is held)
    /// * `callback` - The Python callable to invoke for each key event
    ///
    /// # Callback Signature
    /// The callback will be called with: (event_type: u16, key_code: u16, value: u32)
    pub fn run_with_callback(
        &mut self,
        py: Python,
        callback: &PyObject,
    ) -> Result<(), HybridError> {
        // Install panic handler for logging
        // Note: The Drop trait on PyHybridEventLoop handles device cleanup
        let old_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            eprintln!("HYBRID PANIC: {:?}", panic_info);
            eprintln!("HYBRID PANIC: Devices will be released via Drop trait");
            // Call the old hook to maintain default panic behavior
            old_hook(panic_info);
        }));

        self.running.store(true, Ordering::SeqCst);

        loop {
            // Check if we should stop
            if !self.running.load(Ordering::SeqCst) {
                return Ok(());
            }

            // Release GIL during event polling (performance benefit)
            // This allows Python threads to run while Rust waits for keyboard events
            let events = py
                .allow_threads(|| self.event_reader.poll_for_events(100))
                .map_err(|e| {
                    HybridError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Poll failed: {}", e),
                    ))
                })?;

            // Process events with GIL held
            for event in events {
                // Check if we should stop after polling
                if !self.running.load(Ordering::SeqCst) {
                    return Ok(());
                }

                // Only process key events
                if !event.is_key_event() {
                    continue;
                }

                // Call the Python callback: callback(event_type, key_code, value)
                callback
                    .call1(py, (event.event_type, event.key_code, event.value))
                    .map_err(|e| HybridError::Python(format!("Callback failed: {}", e)))?;

                // Check for Python signals (e.g., Ctrl+C)
                if py.check_signals().is_err() {
                    self.running.store(false, Ordering::SeqCst);
                    return Ok(());
                }
            }
        }
    }
}

/// Drop implementation for PyHybridEventLoop
///
/// This is CRITICAL for system safety. When the hybrid event loop panics or exits,
/// the devices MUST be ungrabbed, otherwise the keyboard will remain in an unusable state.
/// The Drop trait guarantees this cleanup runs even during panic unwinding.
#[cfg(feature = "python-runtime")]
impl Drop for PyHybridEventLoop {
    fn drop(&mut self) {
        // Always ungrab devices when the event loop is destroyed
        // This runs during normal return, early return, panic, and explicit exit
        self.event_reader.ungrab_all();
    }
}

// Simple event processor for testing without full Python integration
#[cfg(feature = "python-runtime")]
pub struct SimpleEventProcessor {
    /// Key remapping (simple modmap)
    remap: std::collections::HashMap<u16, u16>,
}

#[cfg(feature = "python-runtime")]
impl SimpleEventProcessor {
    /// Create a new simple processor with optional remapping
    pub fn new() -> Self {
        Self {
            remap: std::collections::HashMap::new(),
        }
    }

    /// Add a key remapping
    pub fn add_remap(&mut self, from: u16, to: u16) {
        self.remap.insert(from, to);
    }

    /// Process a single event and return the transform result
    pub fn process_event(&self, event: &RawInputEvent) -> TransformResult {
        if !event.is_key_event() {
            return TransformResult::Passthrough(event.key());
        }

        let key = event.key();

        // Check for remap
        if let Some(&remapped_code) = self.remap.get(&key.code()) {
            return TransformResult::Remapped(Key::from(remapped_code));
        }

        // Default passthrough
        TransformResult::Passthrough(key)
    }
}

#[cfg(feature = "python-runtime")]
impl Default for SimpleEventProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "python-runtime")]
    fn test_raw_input_event_action() {
        let event = RawInputEvent {
            time_sec: 0,
            time_usec: 0,
            event_type: 1,
            key_code: 30,
            value: 1,
        };
        assert_eq!(event.action(), Action::Press);

        let event_release = RawInputEvent {
            time_sec: 0,
            time_usec: 0,
            event_type: 1,
            key_code: 30,
            value: 0,
        };
        assert_eq!(event_release.action(), Action::Release);
    }

    #[test]
    fn test_raw_input_event_is_key_event() {
        let event = RawInputEvent {
            time_sec: 0,
            time_usec: 0,
            event_type: 1, // EV_KEY
            key_code: 30,
            value: 1,
        };
        assert!(event.is_key_event());

        let non_key_event = RawInputEvent {
            time_sec: 0,
            time_usec: 0,
            event_type: 2, // Not EV_KEY
            key_code: 0,
            value: 0,
        };
        assert!(!non_key_event.is_key_event());
    }
}

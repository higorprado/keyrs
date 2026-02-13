// Keyrs Batch Event Processing
// Read and write events in batches to reduce syscall overhead

#[cfg(feature = "pure-rust")]
use std::vec::Vec;

/// Batch of input events for processing
///
/// Instead of processing events one at a time, we can read multiple
/// events from the device at once and process them as a batch.
#[derive(Debug, Clone)]
pub struct EventBatch<T> {
    events: Vec<T>,
}

impl<T> EventBatch<T> {
    /// Create a new empty batch
    pub fn new() -> Self {
        Self {
            events: Vec::with_capacity(32),
        }
    }

    /// Create a batch with a pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
        }
    }

    /// Add an event to the batch
    pub fn push(&mut self, event: T) {
        self.events.push(event);
    }

    /// Extend the batch with multiple events
    pub fn extend(&mut self, events: impl IntoIterator<Item = T>) {
        self.events.extend(events);
    }

    /// Get the number of events in the batch
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear the batch
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Iterate over the events
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.events.iter()
    }

    /// Get a reference to the underlying events
    pub fn as_slice(&self) -> &[T] {
        &self.events
    }

    /// Consume the batch and return the events
    pub fn into_vec(self) -> Vec<T> {
        self.events
    }
}

impl<T> Default for EventBatch<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsRef<[T]> for EventBatch<T> {
    fn as_ref(&self) -> &[T] {
        &self.events
    }
}

/// Batch size configuration
///
/// These constants control how many events to batch
/// for optimal performance.
pub mod batch_config {
    /// Default batch size for reading events
    pub const DEFAULT_READ_BATCH: usize = 32;

    /// Default batch size for writing events
    pub const DEFAULT_WRITE_BATCH: usize = 32;

    /// Maximum batch size to prevent memory issues
    pub const MAX_BATCH_SIZE: usize = 256;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_batch_new() {
        let batch: EventBatch<u32> = EventBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_event_batch_push() {
        let mut batch = EventBatch::new();
        batch.push(1);
        batch.push(2);
        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_event_batch_extend() {
        let mut batch = EventBatch::new();
        batch.extend(vec![1, 2, 3]);
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_event_batch_clear() {
        let mut batch = EventBatch::new();
        batch.push(1);
        batch.push(2);
        batch.clear();
        assert!(batch.is_empty());
    }

    #[test]
    fn test_event_batch_iter() {
        let mut batch = EventBatch::new();
        batch.extend(vec![1, 2, 3]);
        let sum: u32 = batch.iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_event_batch_with_capacity() {
        let batch: EventBatch<u32> = EventBatch::with_capacity(100);
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_event_batch_into_vec() {
        let mut batch = EventBatch::new();
        batch.extend(vec![1, 2, 3]);
        let vec = batch.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_batch_config_constants() {
        assert!(batch_config::DEFAULT_READ_BATCH > 0);
        assert!(batch_config::DEFAULT_WRITE_BATCH > 0);
        assert!(batch_config::MAX_BATCH_SIZE >= batch_config::DEFAULT_READ_BATCH);
    }
}

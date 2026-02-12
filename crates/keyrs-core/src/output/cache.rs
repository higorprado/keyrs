// Xwaykeyz Output Cache Tracking
// First-write-wins cache for repeat optimization

use crate::Combo;
use crate::Key;

/// Data stored in the output cache
#[derive(Debug, Clone, PartialEq)]
pub enum CacheData {
    /// Passthrough key press
    Passthrough(Key),
    /// Combo output
    Combo(Combo),
    /// Simple key output
    Key(Key),
}

impl CacheData {
    /// Create passthrough cache data
    pub fn passthrough(key: Key) -> Self {
        Self::Passthrough(key)
    }

    /// Create combo cache data
    pub fn combo(combo: Combo) -> Self {
        Self::Combo(combo)
    }

    /// Create key cache data
    pub fn key(key: Key) -> Self {
        Self::Key(key)
    }

    /// Get the type name of this cache data
    pub fn type_name(&self) -> &str {
        match self {
            CacheData::Passthrough(_) => "passthrough",
            CacheData::Combo(_) => "combo",
            CacheData::Key(_) => "key",
        }
    }
}

/// Output cache for repeat optimization
///
/// Only records the FIRST output (first-write-wins) to prevent
/// send_combo() internal calls from overwriting the combo tracking.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputCache {
    last_output: Option<(String, CacheData)>,
}

impl Default for OutputCache {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self { last_output: None }
    }

    /// Record output for cache (first-write-wins)
    ///
    /// Only records if the cache is empty, preventing overwrites.
    pub fn record(&mut self, output_type: &str, data: CacheData) {
        if self.last_output.is_none() {
            self.last_output = Some((output_type.to_string(), data));
        }
    }

    /// Get the cached output if available
    pub fn get(&self) -> Option<(&str, CacheData)> {
        self.last_output
            .as_ref()
            .map(|(t, d)| (t.as_str(), d.clone()))
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.last_output = None;
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.last_output.is_none()
    }

    /// Get the output type if cached
    pub fn get_type(&self) -> Option<&str> {
        self.last_output.as_ref().map(|(t, _)| t.as_str())
    }

    /// Get the data if cached
    pub fn get_data(&self) -> Option<CacheData> {
        self.last_output.as_ref().map(|(_, d)| d.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Modifier;

    #[test]
    fn test_cache_first_write_wins() {
        let mut cache = OutputCache::new();
        let key_a = Key::from(30);

        cache.record("passthrough", CacheData::passthrough(key_a));
        assert_eq!(cache.get_type(), Some("passthrough"));

        // Second write should be ignored (first-write-wins)
        let key_b = Key::from(48);
        cache.record("combo", CacheData::combo(Combo::new(vec![], key_b)));
        assert_eq!(cache.get_type(), Some("passthrough"));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = OutputCache::new();
        let key = Key::from(30);

        cache.record("key", CacheData::key(key));
        assert!(!cache.is_empty());

        cache.clear();
        assert!(cache.is_empty());
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_cache_get() {
        let mut cache = OutputCache::new();
        let key = Key::from(30);

        cache.record("passthrough", CacheData::passthrough(key));

        let (output_type, data) = cache.get().unwrap();
        assert_eq!(output_type, "passthrough");
        assert_eq!(data, CacheData::passthrough(key));
    }

    #[test]
    fn test_cache_data_types() {
        let key_a = Key::from(30);
        let key_b = Key::from(48);
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let combo = Combo::new(vec![ctrl], key_b);

        let passthrough_data = CacheData::passthrough(key_a);
        assert_eq!(passthrough_data.type_name(), "passthrough");

        let combo_data = CacheData::combo(combo.clone());
        assert_eq!(combo_data.type_name(), "combo");

        let key_data = CacheData::key(key_b);
        assert_eq!(key_data.type_name(), "key");
    }

    #[test]
    fn test_cache_empty_by_default() {
        let cache = OutputCache::new();
        assert!(cache.is_empty());
        assert!(cache.get_type().is_none());
        assert!(cache.get_data().is_none());
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_cache_get_data() {
        let mut cache = OutputCache::new();
        let key = Key::from(30);

        cache.record("key", CacheData::key(key));

        let data = cache.get_data().unwrap();
        assert_eq!(data, CacheData::key(key));
    }

    #[test]
    fn test_cache_data_equality() {
        let key = Key::from(30);

        let data1 = CacheData::key(key);
        let data2 = CacheData::key(key);

        assert_eq!(data1, data2);
    }

    #[test]
    fn test_cache_data_combo() {
        let key = Key::from(30);
        let ctrl = Modifier::from_name("CONTROL").unwrap();
        let combo = Combo::new(vec![ctrl], key);

        let data = CacheData::combo(combo.clone());
        assert_eq!(data.type_name(), "combo");
        if let CacheData::Combo(c) = data {
            assert_eq!(c.key(), key);
        } else {
            panic!("Expected CacheData::Combo");
        }
    }
}

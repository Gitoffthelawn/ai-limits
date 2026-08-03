use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Last successful remaining percent per provider + limit name key, used to
/// detect the "100% again" transition. See the "Previous remaining store"
/// section of `docs/notifications/overview.md`.
pub trait PreviousRemainingStore: Send + Sync {
    /// Atomically reads the previous stored remaining percent for `key`, if
    /// any, and updates the stored value to `current`. The read-then-write
    /// pair is atomic so concurrent callers for the same key cannot both
    /// observe the same previous value.
    fn replace(&self, key: &str, current: f64) -> Option<f64>;
}

#[derive(Default, Deserialize, Serialize)]
struct StoredRemainings(HashMap<String, f64>);

/// File-backed store that survives application restarts. Reads happen once at
/// construction; every successful write is persisted immediately so an
/// unexpected exit does not lose the last known value.
pub struct FileRemainingStore {
    path: PathBuf,
    cache: Mutex<HashMap<String, f64>>,
}

impl FileRemainingStore {
    pub fn new(path: PathBuf) -> Self {
        let cache = Self::read(&path).unwrap_or_default();
        Self {
            path,
            cache: Mutex::new(cache),
        }
    }

    fn read(path: &Path) -> Option<HashMap<String, f64>> {
        let bytes = fs::read(path).ok()?;
        let parsed: StoredRemainings = serde_json::from_slice(&bytes).ok()?;
        Some(parsed.0)
    }

    fn persist(&self, map: &HashMap<String, f64>) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bytes) = serde_json::to_vec(&StoredRemainings(map.clone())) {
            let _ = fs::write(&self.path, bytes);
        }
    }
}

impl PreviousRemainingStore for FileRemainingStore {
    fn replace(&self, key: &str, current: f64) -> Option<f64> {
        let Ok(mut cache) = self.cache.lock() else {
            return None;
        };

        let previous = cache.get(key).copied();
        cache.insert(key.to_string(), current);
        self.persist(&cache);
        previous
    }
}

#[cfg(test)]
pub struct InMemoryRemainingStore {
    values: Mutex<HashMap<String, f64>>,
}

#[cfg(test)]
impl InMemoryRemainingStore {
    pub fn new() -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
        }
    }

    pub fn seed(self, key: &str, value: f64) -> Self {
        self.values
            .lock()
            .expect("store lock should not be poisoned")
            .insert(key.to_string(), value);
        self
    }
}

#[cfg(test)]
impl PreviousRemainingStore for InMemoryRemainingStore {
    fn replace(&self, key: &str, current: f64) -> Option<f64> {
        let mut values = self.values.lock().ok()?;
        let previous = values.get(key).copied();
        values.insert(key.to_string(), current);
        previous
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_store_persists_across_instances() {
        let dir = std::env::temp_dir().join(format!(
            "ai-limits-notifications-store-test-{}",
            std::process::id()
        ));
        let path = dir.join("previous-remaining.json");
        let _ = fs::remove_dir_all(&dir);

        {
            let store = FileRemainingStore::new(path.clone());
            assert_eq!(store.replace("codex|5h", 40.0), None);
        }

        let store = FileRemainingStore::new(path);
        assert_eq!(store.replace("codex|5h", 100.0), Some(40.0));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_store_does_not_rewrite_value_on_missing_file() {
        let dir = std::env::temp_dir().join(format!(
            "ai-limits-notifications-store-missing-{}",
            std::process::id()
        ));
        let path = dir.join("does-not-exist.json");
        let _ = fs::remove_dir_all(&dir);

        let store = FileRemainingStore::new(path);
        assert_eq!(store.replace("codex|5h", 40.0), None);

        let _ = fs::remove_dir_all(&dir);
    }
}

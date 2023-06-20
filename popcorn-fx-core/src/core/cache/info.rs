use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
use log::{debug, error, trace};
use serde::{Deserialize, Serialize};

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M";

/// Cache information containing entries for different caches.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheInfo {
    pub entries: HashMap<String, Vec<CacheEntry>>,
}

impl CacheInfo {
    /// Retrieve the cache info for the given cache name and key if it is known.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cache.
    /// * `key` - The key of the cache entry.
    ///
    /// # Returns
    ///
    /// An optional `CacheEntry` if the cache info is found, otherwise `None`.
    pub fn info(&self, name: &str, key: &str) -> Option<CacheEntry> {
        let key = Self::normalize(key);

        if let Some(entries) = self.entries(name) {
            trace!("Retrieving cache info of key {}", key);
            entries.iter()
                .find(|e| e.key == key)
                .cloned()
        } else {
            trace!("Cache name \"{}\" not found, skipping key info retrieval", Self::normalize(name));
            None
        }
    }

    /// Retrieve the known cache entries of the given cache name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cache.
    ///
    /// # Returns
    ///
    /// An optional slice of `CacheEntry` if the cache entries are found, otherwise `None`.
    pub fn entries(&self, name: &str) -> Option<&[CacheEntry]> {
        let name = Self::normalize(name);

        trace!("Retrieving cache entries of \"{}\"", name);
        self.entries.get(name.as_str())
            .map(|e| &e[..])
    }

    /// Add a new entry to the cache.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cache.
    /// * `entry` - The `CacheEntry` to add.
    ///
    /// # Remarks
    ///
    /// This method adds a new cache entry to the cache. If the cache entry already exists in the cache,
    /// it won't be added again.
    pub fn add(&mut self, name: &str, entry: CacheEntry) {
        let name = Self::normalize(name);
        trace!("Adding new cache {} entry {:?}", name, entry);

        match self.entries.get_mut(name.as_str()) {
            None => {
                trace!("Inserting new cache entry {}", name.as_str());
                self.entries.insert(name, vec![entry]);
            }
            Some(entries) => {
                let cloned_key = entry.key.clone();
                if !entries.iter().any(|e| e.key == cloned_key) {
                    entries.push(entry);
                    debug!("Added new cache {} entry {}", name, cloned_key);
                } else {
                    debug!("Cache {} entry {} already exists, new entry won't be added", name, entry.key)
                }
            }
        }
    }

    /// Remove a cache entry from the cache.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cache.
    /// * `key` - The key of the cache entry.
    pub fn remove(&mut self, name: &str, key: &str) {
        let name = Self::normalize(name);
        let key = Self::normalize(key);

        match self.entries.get_mut(name.as_str()) {
            Some(entries) => {
                let position = entries.iter()
                    .position(|e| e.key == key);

                match position {
                    None => trace!("Cache {} entry {} doesn't exist, ignoring remove action", name, key),
                    Some(e) => {
                        entries.remove(e);
                        debug!("Removed cache {} entry {}", name, key);
                    }
                }
            }
            None => trace!("Cache {} entry doesn't exist, ignoring remove action", name)
        }
    }

    /// Retrieve a list of expired cache entries.
    pub fn expired(&self) -> Vec<ExpiredCacheEntry> {
        let expired_entries: Vec<ExpiredCacheEntry> = self.entries
            .iter()
            .flat_map(|(name, entries)| {
                entries
                    .iter()
                    .filter(|e| e.is_expired(&e.expires_after()))
                    .map(move |entry| ExpiredCacheEntry {
                        name: name.clone(),
                        entry: entry.clone(),
                    })
            })
            .collect();

        debug!("Found a total of {} expired entries", expired_entries.len());
        expired_entries
    }

    fn normalize(value: &str) -> String {
        value
            .to_lowercase()
            .replace(' ', "")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpiredCacheEntry {
   pub name: String,
   pub entry: CacheEntry,
}

/// Cache entry containing information about a cache item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub path: String,
    pub expires_after: i64,
    pub created_on: String,
}

impl CacheEntry {
    /// Create a new `CacheEntry` instance.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the cache entry.
    /// * `path` - The path to the cache data on the filesystem.
    /// * `expires_after` - The expiration duration of the cache entry.
    pub fn new(key: &str, path: &str, expires_after: &Duration) -> Self {
        Self {
            key: CacheInfo::normalize(key),
            path: path.to_string(),
            expires_after: expires_after.num_minutes(),
            created_on: Self::now_as_string(),
        }
    }

    /// Get the key of the cache entry.
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// Get the absolute path of the cache entry.
    pub fn absolute_path(&self) -> &str {
        self.path.as_str()
    }

    /// Get the absolute path of the cache entry.
    pub fn path(&self) -> PathBuf {
        PathBuf::from(self.path.as_str())
    }

    /// Get the filename of the cache entry.
    pub fn filename(&self) -> String {
        self.path().file_name()
            .expect("expected a file and not a directory")
            .to_str()
            .expect("string contains invalid UTF-8 sequence")
            .to_string()
    }

    /// Check if the cache entry has expired.
    ///
    /// # Arguments
    ///
    /// * `validity` - The duration of validity for the cache entry.
    ///
    /// # Returns
    ///
    /// `true` if the cache entry has expired, otherwise `false`.
    pub fn is_expired(&self, validity: &Duration) -> bool {
        Local::now() - self.created_on() > validity.clone()
    }

    /// Get the expiration duration of the cache entry.
    pub fn expires_after(&self) -> Duration {
        Duration::minutes(self.expires_after)
    }

    /// Get the creation timestamp of the cache entry.
    pub fn created_on(&self) -> DateTime<Local> {
        trace!("Parsing cache entry creation datetime {}", self.created_on);
        match NaiveDateTime::parse_from_str(self.created_on.as_str(), DATETIME_FORMAT) {
            Ok(e) => {
                Local.from_local_datetime(&e).unwrap()
            }
            Err(e) => {
                error!("Failed to parse cache entry creation value \"{}\", {}", self.created_on, e);
                Local.timestamp_opt(0, 0).unwrap()
            }
        }
    }

    /// Get the current timestamp as a string representation.
    pub fn now_as_string() -> String {
        Local::now().format(DATETIME_FORMAT).to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_info_key_known() {
        init_logger();
        let cache_name = "lorem";
        let key = "ipsum";
        let filename = "my-filename.cache";
        let cache = CacheInfo {
            entries: vec![
                (cache_name.to_string(), vec![CacheEntry {
                    key: key.to_string(),
                    path: filename.to_string(),
                    created_on: "2023-01-01T12:00:00Z".to_string(),
                    expires_after: 200,
                }])
            ].into_iter().collect(),
        };

        if let Some(entry) = cache.info(cache_name, key) {
            assert_eq!(filename, entry.path.as_str())
        } else {
            assert!(false, "expected a cache entry to have been found")
        }
    }

    #[test]
    fn test_info_key_unknown() {
        init_logger();
        let cache_name = "dolor";
        let key = "ipsum";
        let cache = CacheInfo {
            entries: vec![
                (cache_name.to_string(), vec![])
            ].into_iter().collect(),
        };

        assert_eq!(None, cache.info(cache_name, key))
    }

    #[test]
    fn test_is_expired() {
        init_logger();
        let entry = CacheEntry {
            key: "".to_string(),
            path: "".to_string(),
            created_on: "2023-04-01T00:00".to_string(),
            expires_after: 200,
        };

        assert_eq!(true, entry.is_expired(&Duration::days(1)));
        assert_eq!(false, entry.is_expired(&Duration::weeks(2600)));
    }

    #[test]
    fn test_add() {
        init_logger();
        let name = "lorEm";
        let key = "Ipsum::doLor";
        let entry = CacheEntry::new(key, "/tmp/test", &Duration::days(1));
        let mut info = CacheInfo::default();

        info.add(name, entry.clone());
        assert!(info.info(name, key).is_some(), "expected the entry to have been added");

        // verify that we cannot add it twice
        info.add(name, entry);
        if let Some(entries) = info.entries(name) {
            let mut total = 0;

            for e in entries.iter() {
                if e.key.as_str() == CacheInfo::normalize(key) {
                    total += 1;
                }
            }

            assert_eq!(1, total)
        } else {
            assert!(false, "expected the cache name to have been found")
        }
    }

    #[test]
    fn test_remove() {
        init_logger();
        let name = "lorEm";
        let key = "Ipsum::doLor";
        let mut info = CacheInfo::default();

        info.add(name, CacheEntry::new(key, "/tmp/test", &Duration::weeks(1)));
        assert!(info.info(name, key).is_some(), "expected the entry to have been added");

        info.remove(name, key);
        assert_eq!(None, info.info(name, key))
    }

    #[test]
    fn test_filename() {
        init_logger();
        let entry = CacheEntry::new("lorem", "/tmp/my-file.cache", &Duration::days(1));

        assert_eq!("my-file.cache".to_string(), entry.filename())
    }

    #[test]
    fn test_normalize() {
        init_logger();
        let value = "Lorem IpsuM";
        let expected_value = "loremipsum";

        assert_eq!(expected_value, CacheInfo::normalize(value).as_str())
    }

    #[test]
    fn test_expired() {
        init_logger();
        let expired_entry = CacheEntry {
            key: "ipsum".to_string(),
            path: "".to_string(),
            expires_after: 1,
            created_on: "2023-01-01T12:00".to_string(),
        };
        let cache = CacheInfo {
            entries: vec![
                ("lorem".to_string(), vec![
                    expired_entry.clone(),
                    CacheEntry {
                        key: "dolor".to_string(),
                        path: "".to_string(),
                        expires_after: 5,
                        created_on: CacheEntry::now_as_string(),
                    }
                ]),
                ("ipsum".to_string(), vec![
                    CacheEntry {
                        key: "amet".to_string(),
                        path: "".to_string(),
                        expires_after: 99999,
                        created_on: CacheEntry::now_as_string(),
                    }
                ]),
            ].into_iter().collect(),
        };
        let expected_result = ExpiredCacheEntry {
            name: "lorem".to_string(),
            entry: expired_entry,
        };

        assert_eq!(vec![expected_result], cache.expired())
    }
}
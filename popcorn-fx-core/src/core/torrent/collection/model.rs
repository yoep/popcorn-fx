use log::debug;
use serde::{Deserialize, Serialize};

/// The collection information of magnet torrents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Collection {
    /// The stored magnet torrents
    pub torrents: Vec<MagnetInfo>,
}

impl Collection {
    /// Verify if the collection contains the given uri.
    pub fn contains(&self, uri: &str) -> bool {
        self.torrents.iter()
            .any(|e| e.magnet_uri.as_str() == uri)
    }

    /// Insert the given magnet info into the collection.
    /// If the magnet already exists, it will be ignored.
    pub fn insert(&mut self, name: &str, magnet_uri: &str) {
        if self.contains(magnet_uri) {
            debug!("Magnet info already stored for {}", magnet_uri);
            return;
        }

        self.torrents.push(MagnetInfo {
            name: name.to_string(),
            magnet_uri: magnet_uri.to_string(),
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MagnetInfo {
    /// The name of the magnet
    pub name: String,
    /// The magnet uri of the torrent
    pub magnet_uri: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contains_uri_known() {
        let uri = "magnet:?my-magnet-uri";
        let collection = Collection {
            torrents: vec![
                MagnetInfo {
                    name: "lorem".to_string(),
                    magnet_uri: uri.to_string(),
                }
            ]
        };

        let result = collection.contains(uri);

        assert_eq!(true, result)
    }

    #[test]
    fn test_contains_uri_unknown() {
        let uri = "magnet:?my-magnet-uri";
        let collection = Collection {
            torrents: vec![]
        };

        let result = collection.contains(uri);

        assert_eq!(false, result)
    }

    #[test]
    fn test_insert_new_item() {
        let name = "my-info";
        let uri = "magnet:?something-random";
        let mut collection = Collection {
            torrents: vec![]
        };

        collection.insert(name, uri);
        let result = collection.contains(uri);

        assert_eq!(true, result)
    }

    #[test]
    fn test_insert_duplicate_item() {
        let name = "loremIpsum";
        let uri = "magnet:?estla-dolorSummit";
        let mut collection = Collection {
            torrents: vec![]
        };

        collection.insert(name, uri);
        collection.insert(name, uri);
        let result = collection.torrents.iter()
            .filter(|e| e.name.as_str() == name)
            .count();

        assert_eq!(1, result)
    }
}
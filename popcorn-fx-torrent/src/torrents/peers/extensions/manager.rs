use crate::torrents::peers::extensions::Extension;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

/// The peer extension manager is responsible for managing extension of a session.
/// These extension are then used within each peer connection when applicable.
pub struct PeerExtensionManager {
    extensions: HashMap<String, Box<dyn Extension>>,
}

impl PeerExtensionManager {
    pub fn add<S: AsRef<str>>(&mut self, name: S, extension: Box<dyn Extension>) {
        self.extensions.insert(name.as_ref().to_string(), extension);
    }
}

impl Clone for PeerExtensionManager {
    fn clone(&self) -> Self {
        Self {
            extensions: self
                .extensions
                .iter()
                .map(|(name, e)| (name.clone(), e.clone_box()))
                .collect(),
        }
    }
}

impl Debug for PeerExtensionManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let extension_names: Vec<String> = self
            .extensions
            .iter()
            .map(|(name, _)| name.clone())
            .collect();

        f.debug_struct("PeerExtensionManager")
            .field("extensions", &extension_names)
            .finish()
    }
}

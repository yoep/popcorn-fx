use async_trait::async_trait;
use derive_more::Display;
use log::debug;

use crate::core::playlists::{LoadingStrategy, PlaylistItem};

#[derive(Debug, Display, Default)]
#[display(fmt = "Subtitle loading strategy")]
pub struct SubtitleLoadingStrategy {}

#[async_trait]
impl LoadingStrategy for SubtitleLoadingStrategy {
    async fn process(&self, item: PlaylistItem) -> Option<PlaylistItem> {
        debug!("Loading subtitle for playlist item {}", item);
        Some(item)
    }
}
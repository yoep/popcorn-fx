use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::core::loader::task::LoadingTaskContext;
use crate::core::loader::{
    CancellationResult, LoadingData, LoadingError, LoadingEvent, LoadingResult, LoadingState,
    LoadingStrategy, Result, TorrentData,
};
use crate::core::media::{
    Episode, MediaIdentifier, MediaType, MovieDetails, DEFAULT_AUDIO_LANGUAGE,
};
use crate::core::torrents::{
    Torrent, TorrentEvent, TorrentHandle, TorrentInfo, TorrentManager, TorrentState,
};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, error, trace};
use popcorn_fx_torrent::torrent;
use tokio::select;

const MAGNET_PREFIX: &str = "magnet:?";
const TORRENT_EXTENSION: &str = ".torrent";

/// Represent the loading strategy for loading the torrent information from a media item or a magnet link.
#[derive(Display)]
#[display(fmt = "Torrent info loading strategy")]
pub struct TorrentInfoLoadingStrategy {
    torrent_manager: Arc<Box<dyn TorrentManager>>,
}

impl TorrentInfoLoadingStrategy {
    pub fn new(torrent_manager: Arc<Box<dyn TorrentManager>>) -> Self {
        Self { torrent_manager }
    }

    async fn resolve_torrent_info(
        &self,
        url: &str,
        context: &LoadingTaskContext,
    ) -> Result<(Box<dyn Torrent>, TorrentInfo)> {
        context.send_event(LoadingEvent::StateChanged(LoadingState::Starting));
        // create the torrent
        match self.torrent_manager.create(url).await {
            Ok(torrent) => {
                let mut receiver = torrent.subscribe();
                let handle = torrent.handle();
                let mut info_future = self.torrent_manager.info(&handle);

                loop {
                    select! {
                        _ = context.cancelled() => return Err(LoadingError::Cancelled),
                        Some(event) = receiver.recv() => Self::handle_torrent_event(&*event, context),
                        info = &mut info_future => return info
                            .map(|e| (torrent, e))
                            .map_err(|e| LoadingError::TorrentError(e)),
                    }
                }
            }
            Err(e) => {
                error!("Failed to start playlist playback, {}", e);
                Err(LoadingError::TorrentError(e))
            }
        }
    }

    async fn resolve_torrent_file_from_media(
        &self,
        info: &TorrentInfo,
        media: &Box<dyn MediaIdentifier>,
        quality: &str,
    ) -> Result<torrent::File> {
        match media.media_type() {
            MediaType::Movie => media
                .downcast_ref::<MovieDetails>()
                .and_then(|movie| movie.torrents().get(&DEFAULT_AUDIO_LANGUAGE.to_string()))
                .and_then(|media_torrents| media_torrents.get(&quality.to_string()))
                .and_then(|media_torrent| {
                    media_torrent
                        .file()
                        .and_then(|filename| info.by_filename(filename.as_str()))
                        .or(info.largest_file())
                })
                .ok_or(LoadingError::MediaError(format!(
                    "failed to resolve torrent file for {}",
                    media
                ))),
            MediaType::Episode => media
                .downcast_ref::<Episode>()
                .and_then(|episode| {
                    let episode_torrents = episode.torrents();
                    trace!(
                        "Retrieving {} from episode torrents {:?}",
                        quality,
                        episode_torrents
                    );

                    episode_torrents.get(&quality.to_string())
                })
                .and_then(|media_torrent| {
                    media_torrent
                        .file()
                        .and_then(|filename| {
                            trace!("Searching for torrent file by filename {}", filename);
                            info.by_filename(filename.as_str())
                        })
                        .or_else(|| {
                            trace!(
                                "Torrent file by filename not found, using largest file instead"
                            );
                            info.largest_file()
                        })
                })
                .ok_or(LoadingError::MediaError(format!(
                    "failed to resolve torrent file for {} with quality {}",
                    media, quality
                ))),
            _ => Err(LoadingError::MediaError(format!(
                "unsupported media type {}",
                media.media_type()
            ))),
        }
    }

    async fn cancel_torrent(&self, handle: &TorrentHandle) {
        self.torrent_manager.remove(handle).await;
    }

    fn handle_torrent_event(event: &TorrentEvent, context: &LoadingTaskContext) {
        trace!(
            "Loading task {} torrent info loader received torrent event {:?}",
            context,
            event
        );
        if let TorrentEvent::StateChanged(state) = event {
            match state {
                TorrentState::Initializing => {
                    context.send_event(LoadingEvent::StateChanged(LoadingState::Initializing))
                }
                TorrentState::RetrievingMetadata => {
                    context.send_event(LoadingEvent::StateChanged(LoadingState::RetrievingMetadata))
                }
                TorrentState::CheckingFiles => {
                    context.send_event(LoadingEvent::StateChanged(LoadingState::VerifyingFiles))
                }
                _ => {}
            }
        }
    }
}

impl Debug for TorrentInfoLoadingStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentInfoLoadingStrategy")
            .field("torrent_manager", &self.torrent_manager)
            .finish()
    }
}

#[async_trait]
impl LoadingStrategy for TorrentInfoLoadingStrategy {
    async fn process(&self, data: &mut LoadingData, context: &LoadingTaskContext) -> LoadingResult {
        let mut url: Option<String> = None;

        // check if the url is either a torrent Magnet or torrent file
        if let Some(item_url) = data
            .url
            .as_ref()
            .filter(|url| url.starts_with(MAGNET_PREFIX) || url.ends_with(TORRENT_EXTENSION))
            .cloned()
        {
            url = Some(item_url);
        } else {
            debug!(
                "Playlist item url {:?} is not a magnet, torrent loading is skipped",
                data.url
            );
        }

        if let Some(url) = url {
            debug!("Loading torrent information of {}", url);
            let torrent_info = self.resolve_torrent_info(url.as_str(), context).await;

            match torrent_info {
                Ok((torrent, info)) => {
                    if let Some(media) = data.media.as_ref() {
                        if let Some(quality) = data.quality.as_ref() {
                            trace!(
                                "Updating torrent file info for media {} with quality {}",
                                media,
                                quality
                            );
                            match self
                                .resolve_torrent_file_from_media(&info, media, quality.as_str())
                                .await
                            {
                                Ok(torrent_file) => {
                                    debug!("Updating torrent file info to {:?}", torrent_file);
                                    data.torrent_file = Some(torrent_file.filename());
                                }
                                Err(e) => return LoadingResult::Err(e),
                            }
                        }
                    }

                    debug!("Updating torrent info to {:?}", info);
                    data.url = None; // remove the original url as the item has been enhanced with the data of it
                    data.torrent = Some(TorrentData::Torrent(torrent));
                }
                Err(e) => return LoadingResult::Err(e),
            }
        }

        LoadingResult::Ok
    }

    async fn cancel(&self, mut data: LoadingData) -> CancellationResult {
        if let Some(torrent) = data.torrent.take() {
            debug!(
                "Torrent info loader is cancelling torrent {} for loading task",
                torrent.handle()
            );
            self.cancel_torrent(&torrent.handle()).await;
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::media;
    use crate::core::media::ShowOverview;
    use crate::core::playlist::{PlaylistItem, PlaylistMedia};
    use crate::core::torrents::{MockTorrent, MockTorrentManager, TorrentInfo};
    use crate::{create_loading_task, init_logger, recv_timeout};

    use fx_callback::{Callback, MultiThreadedCallback};
    use mockall::predicate::eq;
    use popcorn_fx_torrent::torrent::TorrentFileInfo;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_process_url() {
        init_logger!();
        let magnet_url = "magnet:?MyTorrent";
        let expected_handle = TorrentHandle::new();
        let info = TorrentInfo {
            handle: Default::default(),
            info_hash: String::new(),
            uri: String::new(),
            name: "MyTorrentInfo".to_string(),
            directory_name: None,
            total_files: 0,
            files: vec![],
        };
        let mut data = LoadingData::from(PlaylistItem {
            url: Some(magnet_url.to_string()),
            title: "Foo bar".to_string(),
            caption: Some("Lorem ipsum dolor".to_string()),
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        let (tx_uri, mut rx_uri) = unbounded_channel();
        let (tx_handle, mut rx_handle) = unbounded_channel();
        let manager_info = info.clone();
        let callback = MultiThreadedCallback::new();
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(expected_handle);
        torrent
            .expect_subscribe()
            .returning(move || callback.subscribe());
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager
            .expect_create()
            .times(1)
            .return_once(move |uri| {
                tx_uri.send(uri.to_string()).unwrap();
                Ok(Box::new(torrent))
            });
        torrent_manager.expect_info().returning(move |handle| {
            tx_handle.send(*handle).unwrap();
            Ok(manager_info.clone())
        });
        let task = create_loading_task!();
        let context = task.context();
        let strategy = TorrentInfoLoadingStrategy::new(Arc::new(Box::new(torrent_manager)));

        // process the data, which should load the magnet url into a Torrent
        let result = strategy.process(&mut data, &*context).await;
        if let LoadingResult::Ok = result {
            assert_eq!(None, data.url, "expected url to be None");
            assert!(data.torrent.is_some(), "expected torrent to be Some");
        } else {
            assert!(
                false,
                "expected LoadingResult::Ok, but got {:?} instead",
                result
            );
        }

        let result_url = recv_timeout!(&mut rx_uri, Duration::from_millis(200));
        assert_eq!(
            magnet_url.to_string(),
            result_url,
            "expected the magnet url to match"
        );

        let result_handle = recv_timeout!(&mut rx_handle, Duration::from_millis(200));
        assert_eq!(
            expected_handle, result_handle,
            "expected the torrent handle to match"
        );
    }

    #[tokio::test]
    async fn test_process_media_url() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let filename = "MySecondFile";
        let magnet_url = "magnet:?MyFullShowTorrent";
        let expected_torrent_file_info = torrent::File {
            index: 2,
            torrent_path: PathBuf::from(filename),
            io_path: temp_dir.path().join(filename),
            offset: 0,
            info: TorrentFileInfo {
                length: 25000,
                path: None,
                path_utf8: None,
                md5sum: None,
                attr: None,
                symlink_path: None,
                sha1: None,
            },
            priority: Default::default(),
        };
        let show = ShowOverview {
            imdb_id: "tt000111".to_string(),
            tvdb_id: "".to_string(),
            title: "MyShow".to_string(),
            year: "2013".to_string(),
            num_seasons: 2,
            images: Default::default(),
            rating: None,
        };
        let episode = Episode {
            season: 1,
            episode: 2,
            first_aired: 0,
            title: "MySecondEpisode".to_string(),
            overview: "".to_string(),
            tvdb_id: 0,
            tvdb_id_value: "".to_string(),
            thumb: None,
            torrents: vec![(
                "720p".to_string(),
                media::TorrentInfo::builder()
                    .url("magnet:?MyEpisodeTorrentUrl")
                    .provider("MyProvider")
                    .source("MySource")
                    .title("MyTitle")
                    .quality("720p")
                    .seed(10)
                    .peer(5)
                    .file(filename)
                    .build(),
            )]
            .into_iter()
            .collect(),
        };
        let item = PlaylistItem {
            url: Some(magnet_url.to_string()),
            title: "Lorem ipsum".to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia {
                parent: Some(Box::new(show)),
                media: Some(Box::new(episode)),
            },
            quality: Some("720p".to_string()),
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let info = TorrentInfo {
            handle: Default::default(),
            info_hash: String::new(),
            uri: String::new(),
            name: "MyShowTorrentInfo".to_string(),
            directory_name: None,
            total_files: 2,
            files: vec![
                torrent::File {
                    index: 1,
                    torrent_path: PathBuf::from("MyFirstFile"),
                    io_path: temp_dir.path().join("MyFirstFile"),
                    offset: 0,
                    info: TorrentFileInfo {
                        length: 25000,
                        path: None,
                        path_utf8: None,
                        md5sum: None,
                        attr: None,
                        symlink_path: None,
                        sha1: None,
                    },
                    priority: Default::default(),
                },
                expected_torrent_file_info.clone(),
            ],
        };
        let mut data = LoadingData::from(item);
        let (tx, mut rx) = unbounded_channel();
        let manager_info = info.clone();
        let task = create_loading_task!();
        let context = task.context();
        let callback = MultiThreadedCallback::<TorrentEvent>::new();
        let mut torrent = MockTorrent::new();
        torrent.expect_handle().return_const(TorrentHandle::new());
        torrent
            .expect_subscribe()
            .returning(move || callback.subscribe());
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager
            .expect_create()
            .with(eq(magnet_url))
            .times(1)
            .return_once(move |uri: &str| {
                tx.send(uri.to_string()).unwrap();
                Ok(Box::new(torrent))
            });
        torrent_manager
            .expect_info()
            .returning(move |_| Ok(manager_info.clone()));

        let strategy = TorrentInfoLoadingStrategy::new(Arc::new(Box::new(torrent_manager)));

        let result = strategy.process(&mut data, &*context).await;

        let resolve_url = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(magnet_url.to_string(), resolve_url);

        if let LoadingResult::Ok = result {
            assert!(
                data.torrent_file.is_some(),
                "expected torrent file to be Some"
            );
        } else {
            assert!(
                false,
                "expected LoadingResult::Ok, but got {:?} instead",
                result
            )
        }
    }

    #[tokio::test]
    async fn test_process_non_magnet_url() {
        init_logger!();
        let magnet_url = "https://www.youtube.com/v/qwe5485";
        let item = PlaylistItem {
            url: Some(magnet_url.to_string()),
            title: "Lorem ipsum".to_string(),
            caption: None,
            thumb: None,
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let info = TorrentInfo {
            handle: Default::default(),
            info_hash: String::new(),
            uri: String::new(),
            name: "MyTorrentInfo".to_string(),
            directory_name: None,
            total_files: 0,
            files: vec![],
        };
        let mut data = LoadingData::from(item);
        let manager_info = info.clone();
        let mut torrent_manager = MockTorrentManager::new();
        torrent_manager
            .expect_info()
            .times(0)
            .returning(move |_| Ok(manager_info.clone()));
        let task = create_loading_task!();
        let context = task.context();
        let strategy = TorrentInfoLoadingStrategy::new(Arc::new(Box::new(torrent_manager)));

        let result = strategy.process(&mut data, &*context).await;
        assert_eq!(LoadingResult::Ok, result);
    }
}

use crate::fx::PopcornFX;
use crate::ipc::proto::message::FxMessage;
use crate::ipc::proto::playlist::{
    GetActivePlaylistRequest, GetActivePlaylistResponse, PlayNextPlaylistItemRequest,
    PlayNextPlaylistItemResponse, PlayPlaylistRequest, PlayPlaylistResponse, PlaylistEvent,
    StopPlaylistRequest,
};
use crate::ipc::proto::{message, playlist};
use crate::ipc::{Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use fx_callback::Callback;
use log::error;
use popcorn_fx_core::core::playlist::Playlist;
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct PlaylistMessageHandler {
    instance: Arc<PopcornFX>,
}

impl PlaylistMessageHandler {
    pub fn new(instance: Arc<PopcornFX>, channel: IpcChannel) -> Self {
        let mut receiver = instance.playlist_manager().subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match playlist::PlaylistEvent::try_from(&*event) {
                    Ok(proto_event) => {
                        if let Err(e) = channel.send(proto_event, PlaylistEvent::NAME).await {
                            error!("Failed to send playlist event to channel, {}", e);
                        }
                    }
                    Err(e) => error!("Failed to parse playlist event, {}", e),
                }
            }
        });

        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for PlaylistMessageHandler {
    fn name(&self) -> &str {
        "playlist"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            PlayPlaylistRequest::NAME
                | PlayNextPlaylistItemRequest::NAME
                | StopPlaylistRequest::NAME
                | GetActivePlaylistRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            PlayPlaylistRequest::NAME => {
                let request = PlayPlaylistRequest::parse_from_bytes(&message.payload)?;
                let playlist = request
                    .playlist
                    .as_ref()
                    .map(Playlist::try_from)
                    .transpose()?
                    .ok_or(Error::MissingField)?;

                let handle = self
                    .instance
                    .playlist_manager()
                    .play(playlist)
                    .await
                    .map(|e| message::Handle {
                        handle: e.value(),
                        special_fields: Default::default(),
                    });

                channel
                    .send_reply(
                        &message,
                        PlayPlaylistResponse {
                            handle: MessageField::from(handle),
                            special_fields: Default::default(),
                        },
                        PlayPlaylistResponse::NAME,
                    )
                    .await?;
            }
            PlayNextPlaylistItemRequest::NAME => {
                let handle =
                    self.instance
                        .playlist_manager()
                        .play_next()
                        .await
                        .map(|e| message::Handle {
                            handle: e.value(),
                            special_fields: Default::default(),
                        });

                channel
                    .send_reply(
                        &message,
                        PlayNextPlaylistItemResponse {
                            handle: MessageField::from(handle),
                            special_fields: Default::default(),
                        },
                        PlayNextPlaylistItemResponse::NAME,
                    )
                    .await?;
            }
            StopPlaylistRequest::NAME => {
                self.instance.playlist_manager().stop().await;
            }
            GetActivePlaylistRequest::NAME => {
                let playlist = playlist::Playlist::try_from(
                    &self.instance.playlist_manager().playlist().await,
                )?;

                channel
                    .send_reply(
                        &message,
                        GetActivePlaylistResponse {
                            playlist: MessageField::some(playlist),
                            special_fields: Default::default(),
                        },
                        GetActivePlaylistResponse::NAME,
                    )
                    .await?;
            }
            _ => {
                return Err(Error::UnsupportedMessage(
                    message.message_type().to_string(),
                ))
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::timeout;

    use popcorn_fx_core::core::media::{Episode, Images, Rating, ShowOverview, TorrentInfo};
    use popcorn_fx_core::core::playlist::{PlaylistItem, PlaylistMedia};
    use popcorn_fx_core::init_logger;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_play_playlist_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlaylistMessageHandler::new(instance.clone(), outgoing.clone());

        let response = incoming
            .get(
                PlayPlaylistRequest {
                    playlist: MessageField::some(playlist::Playlist {
                        items: vec![create_playlist_item()],
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                PlayPlaylistRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = timeout!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = PlayPlaylistResponse::parse_from_bytes(&response.payload).unwrap();

        assert_ne!(MessageField::none(), result.handle);
    }

    #[tokio::test]
    async fn test_process_play_next_playlist_item_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlaylistMessageHandler::new(instance.clone(), outgoing.clone());

        let _ = instance
            .playlist_manager()
            .play(Playlist::from_iter(vec![
                PlaylistItem {
                    url: Some("magnet:?TorrentFile1".to_string()),
                    title: "File1".to_string(),
                    caption: None,
                    thumb: None,
                    media: Default::default(),
                    quality: None,
                    auto_resume_timestamp: None,
                    subtitle: Default::default(),
                    torrent: Default::default(),
                },
                PlaylistItem {
                    url: Some("magnet:?TorrentFile2".to_string()),
                    title: "File2".to_string(),
                    caption: None,
                    thumb: None,
                    media: Default::default(),
                    quality: None,
                    auto_resume_timestamp: None,
                    subtitle: Default::default(),
                    torrent: Default::default(),
                },
            ]))
            .await
            .expect("expected a loading handle to have been returned");

        let response = incoming
            .get(
                PlayNextPlaylistItemRequest::default(),
                PlayNextPlaylistItemRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = timeout!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = PlayNextPlaylistItemResponse::parse_from_bytes(&response.payload).unwrap();

        assert_ne!(MessageField::none(), result.handle);
    }

    #[tokio::test]
    async fn test_process_playlist_stop_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlaylistMessageHandler::new(instance.clone(), outgoing.clone());

        incoming
            .send(StopPlaylistRequest::default(), StopPlaylistRequest::NAME)
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );
    }

    #[tokio::test]
    async fn test_process_get_active_playlist_request() {
        init_logger!();
        let playlist_item = PlaylistItem {
            url: Some("magnet:?TorrentFile2".to_string()),
            title: "File2".to_string(),
            caption: Some("MyCaption".to_string()),
            thumb: Some("MyThumb".to_string()),
            media: PlaylistMedia {
                parent: Some(Box::new(create_show())),
                media: Some(Box::new(create_episode())),
            },
            quality: Some("720p".to_string()),
            auto_resume_timestamp: Some(24000),
            subtitle: Default::default(),
            torrent: Default::default(),
        };
        let proto_playlist_item = playlist::playlist::Item::try_from(&playlist_item).unwrap();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = PlaylistMessageHandler::new(instance.clone(), outgoing.clone());

        let _ = instance
            .playlist_manager()
            .play(Playlist::from_iter(vec![
                PlaylistItem {
                    url: Some("magnet:?TorrentFile1".to_string()),
                    title: "File1".to_string(),
                    caption: None,
                    thumb: None,
                    media: Default::default(),
                    quality: None,
                    auto_resume_timestamp: None,
                    subtitle: Default::default(),
                    torrent: Default::default(),
                },
                playlist_item,
            ]))
            .await
            .expect("expected a loading handle to have been returned");

        let response = incoming
            .get(
                GetActivePlaylistRequest::default(),
                GetActivePlaylistRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(250))
            .expect("expected to have received an incoming message");

        let result = handler.process(message, &outgoing).await;
        assert_eq!(
            Ok(()),
            result,
            "expected the message to have been process successfully"
        );

        let response = timeout!(response, Duration::from_millis(250))
            .expect("expected to have received a reply");
        let result = GetActivePlaylistResponse::parse_from_bytes(&response.payload).unwrap();

        assert_ne!(
            MessageField::none(),
            result.playlist,
            "expected a playlist to have been returned"
        );
        assert_eq!(
            &proto_playlist_item,
            result
                .playlist
                .items
                .get(0)
                .expect("expected a playlist item to have been present")
        );
    }

    fn create_playlist_item() -> playlist::playlist::Item {
        playlist::playlist::Item {
            url: "https://localhost/my-video.mp4".to_string(),
            title: "MyItem".to_string(),
            caption: Some("MyCaption".to_string()),
            thumb: Some("http://localhost/my-thumb.jpg".to_string()),
            quality: Some("1080p".to_string()),
            parent_media: Default::default(),
            media: Default::default(),
            auto_resume_timestamp: Some(10000),
            subtitles_enabled: true,
            torrent_filename: Some("MyItemTorrentFilename".to_string()),
            special_fields: Default::default(),
        }
    }

    fn create_show() -> ShowOverview {
        ShowOverview {
            imdb_id: "ImdbId".to_string(),
            tvdb_id: "TvdbId".to_string(),
            title: "MyShow".to_string(),
            year: "2020".to_string(),
            num_seasons: 5,
            images: Images {
                poster: "ShowPoster".to_string(),
                fanart: "ShowFanart".to_string(),
                banner: "ShowBanner".to_string(),
            },
            rating: Some(Rating {
                percentage: 89,
                watching: 76,
                votes: 42,
                loved: 37,
                hated: 0,
            }),
        }
    }

    fn create_episode() -> Episode {
        Episode {
            season: 4,
            episode: 8,
            first_aired: 0,
            title: "EpisodeTitle".to_string(),
            overview: "EpisodeOverview".to_string(),
            tvdb_id: 0,
            tvdb_id_value: "TvdbId".to_string(),
            thumb: Some("MyEpisodeThumb".to_string()),
            torrents: vec![(
                "720p".to_string(),
                TorrentInfo {
                    url: "magnet:?MyTorrentMagnet".to_string(),
                    provider: "TorrentProvider".to_string(),
                    source: "TorrentSource".to_string(),
                    title: "TorrentTitle".to_string(),
                    quality: " 720p".to_string(),
                    seed: 76,
                    peer: 13,
                    size: Some("100MB".to_string()),
                    filesize: Some("102400".to_string()),
                    file: Some("TorrentFilename".to_string()),
                },
            )]
            .into_iter()
            .collect(),
        }
    }
}

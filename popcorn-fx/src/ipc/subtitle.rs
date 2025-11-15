use crate::fx::PopcornFX;
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::proto::subtitle::{
    subtitle, CleanSubtitlesDirectoryRequest, DownloadAndParseSubtitleRequest,
    DownloadAndParseSubtitleResponse, GetDefaultSubtitlesRequest, GetDefaultSubtitlesResponse,
    GetFileAvailableSubtitlesRequest, GetFileAvailableSubtitlesResponse,
    GetMediaAvailableSubtitlesRequest, GetMediaAvailableSubtitlesResponse,
    GetPreferredSubtitleRequest, GetPreferredSubtitleResponse, GetSubtitlePreferenceRequest,
    GetSubtitlePreferenceResponse, ResetSubtitleRequest, UpdateSubtitlePreferenceRequest,
};
use crate::ipc::{proto, Error, IpcChannel, MessageHandler};
use async_trait::async_trait;
use itertools::Itertools;
use popcorn_fx_core::core::media::{
    Episode, MediaIdentifier, MediaOverview, MovieDetails, ShowDetails,
};
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
use popcorn_fx_core::core::subtitles::{Result, SubtitlePreference};
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct SubtitleMessageHandler {
    instance: Arc<PopcornFX>,
}

impl SubtitleMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }
}

#[async_trait]
impl MessageHandler for SubtitleMessageHandler {
    fn name(&self) -> &str {
        "subtitle"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            GetSubtitlePreferenceRequest::NAME
                | UpdateSubtitlePreferenceRequest::NAME
                | GetDefaultSubtitlesRequest::NAME
                | GetMediaAvailableSubtitlesRequest::NAME
                | GetFileAvailableSubtitlesRequest::NAME
                | GetPreferredSubtitleRequest::NAME
                | DownloadAndParseSubtitleRequest::NAME
                | ResetSubtitleRequest::NAME
                | CleanSubtitlesDirectoryRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> crate::ipc::Result<()> {
        match message.message_type() {
            GetSubtitlePreferenceRequest::NAME => {
                let mut response = GetSubtitlePreferenceResponse::new();
                let preference = self.instance.subtitle_manager().preference().await;
                response.preference =
                    MessageField::some(proto::subtitle::SubtitlePreference::from(&preference));

                channel
                    .send_reply(&message, response, GetSubtitlePreferenceResponse::NAME)
                    .await?;
            }
            UpdateSubtitlePreferenceRequest::NAME => {
                let request = UpdateSubtitlePreferenceRequest::parse_from_bytes(&message.payload)?;
                let preference = request
                    .preference
                    .as_ref()
                    .map(SubtitlePreference::try_from)
                    .transpose()?
                    .ok_or(Error::MissingField)?;

                self.instance
                    .subtitle_manager()
                    .update_preference(preference)
                    .await;
            }
            GetDefaultSubtitlesRequest::NAME => {
                channel
                    .send_reply(
                        &message,
                        GetDefaultSubtitlesResponse {
                            subtitles: vec![SubtitleInfo::none(), SubtitleInfo::custom()]
                                .iter()
                                .map(subtitle::Info::from)
                                .collect(),
                            special_fields: Default::default(),
                        },
                        GetDefaultSubtitlesResponse::NAME,
                    )
                    .await?;
            }
            GetMediaAvailableSubtitlesRequest::NAME => {
                let request =
                    GetMediaAvailableSubtitlesRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaOverview>::try_from(
                    request.item.as_ref().ok_or(Error::MissingField)?,
                )?;
                let response: GetMediaAvailableSubtitlesResponse;
                let mut search_result: Option<Result<Vec<SubtitleInfo>>> = None;

                if let Some(movie) = media.downcast_ref::<MovieDetails>() {
                    search_result = Some(
                        self.instance
                            .subtitle_provider()
                            .movie_subtitles(movie)
                            .await,
                    );
                } else if let Some(show) = media.downcast_ref::<ShowDetails>() {
                    let sub_item = Box::<dyn MediaIdentifier>::try_from(
                        request.sub_item.as_ref().ok_or(Error::MissingField)?,
                    )?;

                    if let Some(episode) = sub_item.downcast_ref::<Episode>() {
                        search_result = Some(
                            self.instance
                                .subtitle_provider()
                                .episode_subtitles(show, episode)
                                .await,
                        );
                    }
                }

                if let Some(result) = search_result {
                    match result {
                        Ok(subtitles) => {
                            response = GetMediaAvailableSubtitlesResponse {
                                result: response::Result::OK.into(),
                                subtitles: subtitles.iter().map(subtitle::Info::from).collect(),
                                error: Default::default(),
                                special_fields: Default::default(),
                            };
                        }
                        Err(err) => {
                            response = GetMediaAvailableSubtitlesResponse {
                                result: response::Result::ERROR.into(),
                                subtitles: vec![],
                                error: MessageField::some(subtitle::Error::from(&err)),
                                special_fields: Default::default(),
                            };
                        }
                    }
                } else {
                    response = GetMediaAvailableSubtitlesResponse {
                        result: response::Result::ERROR.into(),
                        subtitles: vec![],
                        error: MessageField::some(subtitle::Error {
                            type_: subtitle::error::Type::SEARCH_FAILED.into(),
                            invalid_url: Default::default(),
                            search_failed: MessageField::some(subtitle::error::SearchFailed {
                                reason: format!(
                                    "Media item type \"{}\" is not supported",
                                    request.item.type_
                                ),
                                special_fields: Default::default(),
                            }),
                            download_failed: Default::default(),
                            conversion_failed: Default::default(),
                            unsupported_type: Default::default(),
                            special_fields: Default::default(),
                        }),
                        special_fields: Default::default(),
                    };
                }

                channel
                    .send_reply(&message, response, GetMediaAvailableSubtitlesResponse::NAME)
                    .await?;
            }
            GetFileAvailableSubtitlesRequest::NAME => {
                let request = GetFileAvailableSubtitlesRequest::parse_from_bytes(&message.payload)?;
                let response: GetFileAvailableSubtitlesResponse;

                match self
                    .instance
                    .subtitle_provider()
                    .file_subtitles(request.filename.as_str())
                    .await
                {
                    Ok(subtitles) => {
                        response = GetFileAvailableSubtitlesResponse {
                            result: response::Result::OK.into(),
                            subtitles: subtitles.iter().map(subtitle::Info::from).collect(),
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(err) => {
                        response = GetFileAvailableSubtitlesResponse {
                            result: response::Result::ERROR.into(),
                            subtitles: vec![],
                            error: MessageField::some(subtitle::Error::from(&err)),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, GetFileAvailableSubtitlesResponse::NAME)
                    .await?;
            }
            GetPreferredSubtitleRequest::NAME => {
                let request = GetPreferredSubtitleRequest::parse_from_bytes(&message.payload)?;
                let subtitles: Vec<SubtitleInfo> = request
                    .subtitles
                    .iter()
                    .map(SubtitleInfo::try_from)
                    .try_collect()?;

                let subtitle = self
                    .instance
                    .subtitle_manager()
                    .select_or_default(&subtitles)
                    .await;

                channel
                    .send_reply(
                        &message,
                        GetPreferredSubtitleResponse {
                            subtitle: MessageField::some(subtitle::Info::from(&subtitle)),
                            special_fields: Default::default(),
                        },
                        GetPreferredSubtitleResponse::NAME,
                    )
                    .await?;
            }
            DownloadAndParseSubtitleRequest::NAME => {
                let request = DownloadAndParseSubtitleRequest::parse_from_bytes(&message.payload)?;
                let info = request
                    .info
                    .as_ref()
                    .map(SubtitleInfo::try_from)
                    .transpose()?
                    .ok_or(Error::MissingField)?;
                let matcher = request
                    .matcher
                    .as_ref()
                    .map(SubtitleMatcher::from)
                    .ok_or(Error::MissingField)?;
                let response: DownloadAndParseSubtitleResponse;

                match self
                    .instance
                    .subtitle_provider()
                    .download_and_parse(&info, &matcher)
                    .await
                {
                    Ok(subtitle) => {
                        response = DownloadAndParseSubtitleResponse {
                            result: response::Result::OK.into(),
                            subtitle: MessageField::some(proto::subtitle::Subtitle::from(
                                &subtitle,
                            )),
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(e) => {
                        response = DownloadAndParseSubtitleResponse {
                            result: response::Result::ERROR.into(),
                            subtitle: Default::default(),
                            error: MessageField::some(subtitle::Error::from(&e)),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, DownloadAndParseSubtitleResponse::NAME)
                    .await?;
            }
            ResetSubtitleRequest::NAME => {
                self.instance.subtitle_manager().reset().await;
            }
            CleanSubtitlesDirectoryRequest::NAME => {
                self.instance.subtitle_manager().cleanup().await;
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

    use crate::ipc::proto::media::media;
    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::timeout;

    use httpmock::{Method, MockServer};
    use popcorn_fx_core::core::media::{Images, Rating, TorrentInfo};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::read_test_file_to_string;
    use protobuf::EnumOrUnknown;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_get_subtitle_preference_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        instance
            .subtitle_manager()
            .update_preference(SubtitlePreference::Language(SubtitleLanguage::German))
            .await;

        let response = incoming
            .get(
                GetSubtitlePreferenceRequest::new(),
                GetSubtitlePreferenceRequest::NAME,
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
        let result = GetSubtitlePreferenceResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            MessageField::some(proto::subtitle::SubtitlePreference {
                preference: proto::subtitle::subtitle_preference::Preference::LANGUAGE.into(),
                language: Some(subtitle::Language::GERMAN.into()),
                special_fields: Default::default(),
            }),
            result.preference
        );
    }

    #[tokio::test]
    async fn test_process_update_subtitle_preference_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        incoming
            .send(
                UpdateSubtitlePreferenceRequest {
                    preference: MessageField::some(proto::subtitle::SubtitlePreference {
                        preference: proto::subtitle::subtitle_preference::Preference::LANGUAGE
                            .into(),
                        language: Some(subtitle::Language::BULGARIAN.into()),
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                UpdateSubtitlePreferenceRequest::NAME,
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

        let result = instance.subtitle_manager().preference().await;
        assert_eq!(
            SubtitlePreference::Language(SubtitleLanguage::Bulgarian),
            result
        );
    }

    #[tokio::test]
    async fn test_process_get_default_subtitles_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        let response = incoming
            .get(
                GetDefaultSubtitlesRequest::new(),
                GetDefaultSubtitlesRequest::NAME,
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
        let result = GetDefaultSubtitlesResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            2,
            result.subtitles.len(),
            "expected 2 subtitles to have been returned"
        );
    }

    #[tokio::test]
    async fn test_process_get_media_episode_available_subtitles_request() {
        init_logger!();
        let media = Box::new(ShowDetails {
            title: "MyShow".to_string(),
            imdb_id: "tt31589662".to_string(),
            tvdb_id: "448023".to_string(),
            year: "2024".to_string(),
            num_seasons: 2,
            images: Images {
                poster: "http://localhost/poster.png".to_string(),
                fanart: "http://localhost/fanart.png".to_string(),
                banner: "http://localhost/banner.png".to_string(),
            },
            rating: Some(Rating {
                percentage: 80,
                watching: 20,
                votes: 0,
                loved: 0,
                hated: 0,
            }),
            context_locale: "".to_string(),
            synopsis: "".to_string(),
            runtime: None,
            status: "".to_string(),
            genres: vec![],
            episodes: vec![],
        }) as Box<dyn MediaIdentifier>;
        let episode = Box::new(Episode {
            season: 1,
            episode: 1,
            first_aired: 0,
            title: "".to_string(),
            overview: "".to_string(),
            tvdb_id: 10400707,
            tvdb_id_value: "10400707".to_string(),
            thumb: None,
            torrents: Default::default(),
        }) as Box<dyn MediaIdentifier>;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        let response = incoming
            .get(
                GetMediaAvailableSubtitlesRequest {
                    item: MessageField::some(media::Item::try_from(&media).unwrap()),
                    sub_item: MessageField::some(media::Item::try_from(&episode).unwrap()),
                    special_fields: Default::default(),
                },
                GetMediaAvailableSubtitlesRequest::NAME,
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
        let result =
            GetMediaAvailableSubtitlesResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            EnumOrUnknown::from(response::Result::OK),
            result.result,
            "expected response result OK, got {:?} instead",
            result
        );
        assert_ne!(Vec::<subtitle::Info>::new(), result.subtitles);
    }

    #[tokio::test]
    async fn test_process_get_media_movie_available_subtitles_request() {
        init_logger!();
        let imdb_id = "tt1156398";
        let response = read_test_file_to_string("subtitles-movie.json");
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(Method::GET)
                .path("/api/v1/subtitles")
                .query_param("imdb_id", imdb_id.replace("tt", ""));
            then.status(200).body(response);
        });
        let media = Box::new(MovieDetails {
            title: "MyShow".to_string(),
            imdb_id: imdb_id.to_string(),
            year: "2009".to_string(),
            images: Images {
                poster: "http://localhost/poster.png".to_string(),
                fanart: "http://localhost/fanart.png".to_string(),
                banner: "http://localhost/banner.png".to_string(),
            },
            trailer: "https://www.youtube.com/watch?v=8m9EVP8X7N8".to_string(),
            rating: Some(Rating {
                percentage: 72,
                watching: 31,
                votes: 28773,
                loved: 0,
                hated: 0,
            }),
            synopsis: "".to_string(),
            genres: vec!["comedy".to_string()],
            runtime: "88".to_string(),
            torrents: vec![(
                "en".to_string(),
                vec![(
                    "720p".to_string(),
                    TorrentInfo {
                        url: "TorrentUrl".to_string(),
                        provider: "TorrentProvider".to_string(),
                        source: "TorrentSource".to_string(),
                        title: "TorrentTitle".to_string(),
                        quality: "720p".to_string(),
                        seed: 687,
                        peer: 89,
                        size: None,
                        filesize: None,
                        file: None,
                    },
                )]
                .into_iter()
                .collect(),
            )]
            .into_iter()
            .collect(),
        }) as Box<dyn MediaIdentifier>;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut args = default_args(temp_path);
        args.properties.subtitle.url = server.url("/api/v1");
        let instance = Arc::new(PopcornFX::new(args).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        let response = incoming
            .get(
                GetMediaAvailableSubtitlesRequest {
                    item: MessageField::some(media::Item::try_from(&media).unwrap()),
                    sub_item: Default::default(),
                    special_fields: Default::default(),
                },
                GetMediaAvailableSubtitlesRequest::NAME,
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
        let result =
            GetMediaAvailableSubtitlesResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            EnumOrUnknown::from(response::Result::OK),
            result.result,
            "expected response Ok, but got {:?} instead",
            result
        );
        assert_ne!(Vec::<subtitle::Info>::new(), result.subtitles);
    }

    #[tokio::test]
    async fn test_process_get_file_available_subtitles_request() {
        init_logger!();
        let filename = "Zombieland";
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        let response = incoming
            .get(
                GetFileAvailableSubtitlesRequest {
                    filename: filename.to_string(),
                    special_fields: Default::default(),
                },
                GetFileAvailableSubtitlesRequest::NAME,
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
        let result =
            GetFileAvailableSubtitlesResponse::parse_from_bytes(&response.payload).unwrap();

        let response_result = result.result.unwrap();
        assert_eq!(
            response::Result::OK,
            response_result,
            "expected response result OK, got {:?} instead",
            result
        );
        assert_ne!(Vec::<subtitle::Info>::new(), result.subtitles);
    }

    #[tokio::test]
    async fn test_process_preferred_subtitle_request() {
        init_logger!();
        let subtitle_info = subtitle::Info {
            imdb_id: Some("tt1156398".to_string()),
            language: subtitle::Language::ENGLISH.into(),
            files: vec![subtitle::info::File {
                file_id: 19800,
                name: "SubtitleFile".to_string(),
                url: "SubtitleUrl".to_string(),
                score: 9.8,
                downloads: 19876,
                quality: Some(720),
                special_fields: Default::default(),
            }],
            special_fields: Default::default(),
        };
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        instance
            .subtitle_manager()
            .update_preference(SubtitlePreference::Language(SubtitleLanguage::English))
            .await;

        let response = incoming
            .get(
                GetPreferredSubtitleRequest {
                    subtitles: vec![
                        subtitle_info.clone(),
                        subtitle::Info {
                            imdb_id: Some("tt1156398".to_string()),
                            language: subtitle::Language::FRENCH.into(),
                            files: vec![],
                            special_fields: Default::default(),
                        },
                    ],
                    special_fields: Default::default(),
                },
                GetPreferredSubtitleRequest::NAME,
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
        let result = GetPreferredSubtitleResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(MessageField::some(subtitle_info), result.subtitle);
    }

    #[tokio::test]
    async fn test_process_download_and_parse_subtitle_request() {
        init_logger!();
        let subtitle_info = subtitle::Info {
            imdb_id: Some("tt1156398".to_string()),
            language: subtitle::Language::ENGLISH.into(),
            files: vec![subtitle::info::File {
                file_id: 4946154,
                name: "Zombieland.2009.BluRay.srt".to_string(),
                url: "https://www.opensubtitles.com/ar/subtitles/legacy/7739662".to_string(),
                score: 10.0,
                downloads: 17198,
                quality: Some(720),
                special_fields: Default::default(),
            }],
            special_fields: Default::default(),
        };
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        instance
            .subtitle_manager()
            .update_preference(SubtitlePreference::Language(SubtitleLanguage::English))
            .await;

        let response = incoming
            .get(
                DownloadAndParseSubtitleRequest {
                    info: MessageField::some(subtitle_info.clone()),
                    matcher: MessageField::some(subtitle::Matcher {
                        filename: "SomeMovie".to_string(),
                        quality: Some("720p".to_string()),
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                },
                DownloadAndParseSubtitleRequest::NAME,
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
        let result = DownloadAndParseSubtitleResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(
            response::Result::OK,
            result.result.unwrap(),
            "expected Result::OK, but got {:?} instead",
            result
        );
        assert_eq!(MessageField::some(subtitle_info), result.subtitle.info);
        assert_ne!(
            "", &result.subtitle.file_path,
            "expected the the subtitle to have been downloaded"
        );
        assert_ne!(
            Vec::<subtitle::Cue>::new(),
            result.subtitle.cues,
            "expected the subtitle to have been parsed"
        );
    }

    #[tokio::test]
    async fn test_process_reset_subtitle_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        incoming
            .send(ResetSubtitleRequest::new(), ResetSubtitleRequest::NAME)
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
    async fn test_process_clean_subtitles_directory_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = SubtitleMessageHandler::new(instance.clone());

        incoming
            .send(
                CleanSubtitlesDirectoryRequest::new(),
                CleanSubtitlesDirectoryRequest::NAME,
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
    }
}

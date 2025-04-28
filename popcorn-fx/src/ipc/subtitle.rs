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

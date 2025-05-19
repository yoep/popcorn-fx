use crate::fx::PopcornFX;
use crate::ipc::proto::media::media;
use crate::ipc::proto::media::{
    get_category_genres_response, get_category_sort_by_response, GetCategoryGenresRequest,
    GetCategoryGenresResponse, GetCategorySortByRequest, GetCategorySortByResponse,
    GetMediaDetailsRequest, GetMediaDetailsResponse, GetMediaItemsRequest, GetMediaItemsResponse,
    ResetProviderApiRequest,
};
use crate::ipc::proto::message::{response, FxMessage};
use crate::ipc::{Error, IpcChannel, MessageHandler, Result};
use async_trait::async_trait;
use log::warn;
use popcorn_fx_core::core::config::{ConfigError, ProviderProperties};
use popcorn_fx_core::core::media::{Category, Genre, MediaIdentifier, MediaOverview, SortBy};
use protobuf::{Message, MessageField};
use std::sync::Arc;

#[derive(Debug)]
pub struct MediaMessageHandler {
    instance: Arc<PopcornFX>,
}

impl MediaMessageHandler {
    pub fn new(instance: Arc<PopcornFX>) -> Self {
        Self { instance }
    }

    /// Try to get the provider information of the given category.
    fn category_provider(
        &self,
        category: &media::Category,
    ) -> std::result::Result<ProviderProperties, ConfigError> {
        self.instance
            .settings()
            .properties_ref()
            .provider(String::from(category))
            .cloned()
    }
}

#[async_trait]
impl MessageHandler for MediaMessageHandler {
    fn name(&self) -> &str {
        "media"
    }

    fn is_supported(&self, message_type: &str) -> bool {
        matches!(
            message_type,
            GetCategoryGenresRequest::NAME
                | GetCategorySortByRequest::NAME
                | GetMediaDetailsRequest::NAME
                | GetMediaItemsRequest::NAME
                | ResetProviderApiRequest::NAME
        )
    }

    async fn process(&self, message: FxMessage, channel: &IpcChannel) -> Result<()> {
        match message.message_type() {
            GetCategoryGenresRequest::NAME => {
                let request = GetCategoryGenresRequest::parse_from_bytes(&message.payload)?;
                let mut response = GetCategoryGenresResponse::new();

                match request.category.enum_value() {
                    Ok(category) => {
                        match self.category_provider(&category).map(|e| e.genres.clone()) {
                            Ok(genres) => {
                                response.result = response::Result::OK.into();
                                response.genres = genres
                                    .into_iter()
                                    .map(|e| {
                                        let mut genre = media::Genre::new();
                                        genre.key = e;
                                        genre
                                    })
                                    .collect();
                            }
                            Err(e) => {
                                warn!("Category provider not found, {}", e);
                                response.result = response::Result::ERROR.into();
                                response.error = Some(
                                    get_category_genres_response::Error::PROVIDER_NOT_FOUND.into(),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Category enum value {} is invalid", e);
                        response.result = response::Result::ERROR.into();
                        response.error =
                            Some(get_category_genres_response::Error::INVALID_CATEGORY.into());
                    }
                }

                channel
                    .send_reply(&message, response, GetCategoryGenresResponse::NAME)
                    .await?;
            }
            GetCategorySortByRequest::NAME => {
                let request = GetCategorySortByRequest::parse_from_bytes(&message.payload)?;
                let mut response = GetCategorySortByResponse::new();

                match request.category.enum_value() {
                    Ok(category) => {
                        match self.category_provider(&category).map(|e| e.sort_by.clone()) {
                            Ok(sort_by) => {
                                response.result = response::Result::OK.into();
                                response.sort_by = sort_by
                                    .into_iter()
                                    .map(|e| {
                                        let mut sort = media::SortBy::new();
                                        sort.key = e;
                                        sort
                                    })
                                    .collect();
                            }
                            Err(e) => {
                                warn!("Category provider not found, {}", e);
                                response.result = response::Result::ERROR.into();
                                response.error = Some(
                                    get_category_sort_by_response::Error::PROVIDER_NOT_FOUND.into(),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Category enum value {} is invalid", e);
                        response.result = response::Result::ERROR.into();
                        response.error =
                            Some(get_category_sort_by_response::Error::INVALID_CATEGORY.into());
                    }
                }

                channel
                    .send_reply(&message, response, GetCategorySortByResponse::NAME)
                    .await?;
            }
            GetMediaItemsRequest::NAME => {
                let request = GetMediaItemsRequest::parse_from_bytes(&message.payload)?;
                let category = request
                    .category
                    .enum_value()
                    .map(|e| Category::from(&e))
                    .map_err(|_| Error::UnsupportedEnum)?;
                let genre = request
                    .genre
                    .as_ref()
                    .map(Genre::from)
                    .ok_or(Error::MissingField)?;
                let sort_by = request
                    .sort_by
                    .as_ref()
                    .map(SortBy::from)
                    .ok_or(Error::MissingField)?;
                let keywords = request.keywords.clone().unwrap_or_default();
                let response: GetMediaItemsResponse;

                let result = self
                    .instance
                    .providers()
                    .retrieve(&category, &genre, &sort_by, &keywords, request.page)
                    .await;

                match result {
                    Ok(items) => {
                        let items = items
                            .into_iter()
                            .map(|e| media::Item::try_from(&(e as Box<dyn MediaIdentifier>)))
                            .collect::<Result<Vec<media::Item>>>()?;

                        response = GetMediaItemsResponse {
                            result: response::Result::OK.into(),
                            items,
                            error: Default::default(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(media_error) => {
                        warn!("Failed to retrieve category media items, {}", media_error);
                        response = GetMediaItemsResponse {
                            result: response::Result::ERROR.into(),
                            items: vec![],
                            error: MessageField::some(media::Error::from(&media_error)),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, GetMediaItemsResponse::NAME)
                    .await?;
            }
            GetMediaDetailsRequest::NAME => {
                let request = GetMediaDetailsRequest::parse_from_bytes(&message.payload)?;
                let media = Box::<dyn MediaOverview>::try_from(
                    request.item.as_ref().ok_or(Error::MissingField)?,
                )?;
                let response: GetMediaDetailsResponse;

                match self
                    .instance
                    .providers()
                    .retrieve_details(&(media as Box<dyn MediaIdentifier>))
                    .await
                {
                    Ok(media) => {
                        response = GetMediaDetailsResponse {
                            result: response::Result::OK.into(),
                            item: MessageField::some(media::Item::try_from(
                                &(media as Box<dyn MediaIdentifier>),
                            )?),
                            error: MessageField::none(),
                            special_fields: Default::default(),
                        };
                    }
                    Err(err) => {
                        response = GetMediaDetailsResponse {
                            result: response::Result::ERROR.into(),
                            item: Default::default(),
                            error: MessageField::some(media::Error::from(&err)),
                            special_fields: Default::default(),
                        };
                    }
                }

                channel
                    .send_reply(&message, response, GetMediaDetailsResponse::NAME)
                    .await?;
            }
            ResetProviderApiRequest::NAME => {
                let request = ResetProviderApiRequest::parse_from_bytes(&message.payload)?;
                let category = request
                    .category
                    .enum_value()
                    .map(|e| Category::from(&e))
                    .map_err(|_| Error::UnsupportedEnum)?;

                self.instance.providers().reset_api(&category).await;
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

    use crate::ipc::proto::media::media::MovieDetails;
    use crate::ipc::test::create_channel_pair;
    use crate::tests::default_args;
    use crate::timeout;

    use popcorn_fx_core::core::media::{Images, MovieOverview, Rating};
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::copy_test_file;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_category_genres_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(default_args(temp_path)).await.unwrap();
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = MediaMessageHandler::new(Arc::new(instance));

        let response = incoming
            .get(
                GetCategoryGenresRequest {
                    category: media::Category::MOVIES.into(),
                    special_fields: Default::default(),
                },
                GetCategoryGenresRequest::NAME,
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
        let result = GetCategoryGenresResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(response::Result::OK, result.result.unwrap());
        assert_ne!(0, result.genres.len(), "expected to receive genres");
    }

    #[tokio::test]
    async fn test_process_category_sort_by_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(default_args(temp_path)).await.unwrap();
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = MediaMessageHandler::new(Arc::new(instance));

        let response = incoming
            .get(
                GetCategorySortByRequest {
                    category: media::Category::SERIES.into(),
                    special_fields: Default::default(),
                },
                GetCategorySortByRequest::NAME,
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
        let result = GetCategorySortByResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(response::Result::OK, result.result.unwrap());
        assert_ne!(
            0,
            result.sort_by.len(),
            "expected to receive sort by options"
        );
    }

    #[tokio::test]
    async fn test_process_get_media_items_request_favorites() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "favorites.json", None);
        let instance = PopcornFX::new(default_args(temp_path)).await.unwrap();
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = MediaMessageHandler::new(Arc::new(instance));

        let response = incoming
            .get(
                GetMediaItemsRequest {
                    category: media::Category::FAVORITES.into(),
                    genre: MessageField::some(media::Genre {
                        key: "all".to_string(),
                        text: "All".to_string(),
                        special_fields: Default::default(),
                    }),
                    sort_by: MessageField::some(media::SortBy {
                        key: "watched".to_string(),
                        text: "Watched".to_string(),
                        special_fields: Default::default(),
                    }),
                    keywords: None,
                    page: 0,
                    special_fields: Default::default(),
                },
                GetMediaItemsRequest::NAME,
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
        let result = GetMediaItemsResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(response::Result::OK, result.result.unwrap());
    }

    #[tokio::test]
    async fn test_process_get_media_details_request() {
        init_logger!();
        let media = Box::new(MovieOverview {
            title: "MyMovie".to_string(),
            imdb_id: "tt1156398".to_string(),
            year: "2013".to_string(),
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
        }) as Box<dyn MediaIdentifier>;
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = MediaMessageHandler::new(instance);

        let response = incoming
            .get(
                GetMediaDetailsRequest {
                    item: MessageField::some(media::Item::try_from(&media).unwrap()),
                    special_fields: Default::default(),
                },
                GetMediaDetailsRequest::NAME,
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
        let result = GetMediaDetailsResponse::parse_from_bytes(&response.payload).unwrap();

        assert_eq!(response::Result::OK, result.result.unwrap());
        assert_eq!(MovieDetails::NAME, result.item.type_);
        assert_ne!(MessageField::none(), result.item.movie_details);
    }

    #[tokio::test]
    async fn test_process_reset_provider_api_request() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = Arc::new(PopcornFX::new(default_args(temp_path)).await.unwrap());
        let (incoming, outgoing) = create_channel_pair().await;
        let handler = MediaMessageHandler::new(instance);

        incoming
            .send(
                ResetProviderApiRequest {
                    category: media::Category::MOVIES.into(),
                    special_fields: Default::default(),
                },
                ResetProviderApiRequest::NAME,
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

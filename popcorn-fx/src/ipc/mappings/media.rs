use crate::ipc::proto::media::media;
use crate::ipc::{Error, Result};
use popcorn_fx_core::core::media::{
    Category, Episode, Genre, Images, MediaDetails, MediaError, MediaIdentifier, MediaOverview,
    MovieDetails, MovieOverview, Rating, ShowDetails, ShowOverview, SortBy, TorrentInfo,
};
use protobuf::{Message, MessageField};
use std::collections::HashMap;

impl From<&Category> for media::Category {
    fn from(value: &Category) -> Self {
        match value {
            Category::Movies => media::Category::MOVIES,
            Category::Series => media::Category::SERIES,
            Category::Favorites => media::Category::FAVORITES,
        }
    }
}

impl From<&media::Category> for Category {
    fn from(value: &media::Category) -> Self {
        match value {
            media::Category::MOVIES => Category::Movies,
            media::Category::SERIES => Category::Series,
            media::Category::FAVORITES => Category::Favorites,
        }
    }
}

impl From<&media::Category> for String {
    fn from(value: &media::Category) -> Self {
        match value {
            media::Category::MOVIES => "movies".to_string(),
            media::Category::SERIES => "series".to_string(),
            media::Category::FAVORITES => "favorites".to_string(),
        }
    }
}

impl From<&media::Genre> for Genre {
    fn from(value: &media::Genre) -> Self {
        Self::new(value.key.clone(), value.text.clone())
    }
}

impl From<&media::SortBy> for SortBy {
    fn from(value: &media::SortBy) -> Self {
        Self::new(value.key.clone(), value.text.clone())
    }
}

impl From<&media::MovieOverview> for MovieOverview {
    fn from(value: &media::MovieOverview) -> Self {
        Self {
            title: value.title.clone(),
            imdb_id: value.imdb_id.clone(),
            year: value.year.clone(),
            rating: value.rating.as_ref().map(Rating::from),
            images: value
                .images
                .as_ref()
                .map(Images::from)
                .unwrap_or(Images::default()),
        }
    }
}

impl From<&MovieOverview> for media::MovieOverview {
    fn from(value: &MovieOverview) -> Self {
        Self {
            title: value.title(),
            imdb_id: value.imdb_id().to_string(),
            year: value.year().to_string(),
            images: MessageField::some(media::Images::from(value.images())),
            rating: value
                .rating
                .as_ref()
                .map(media::Rating::from)
                .map(MessageField::some)
                .unwrap_or_default(),
            special_fields: Default::default(),
        }
    }
}

impl From<&media::ShowOverview> for ShowOverview {
    fn from(value: &media::ShowOverview) -> Self {
        Self {
            imdb_id: value.imdb_id.clone(),
            tvdb_id: value.tvdb_id.clone(),
            title: value.title.clone(),
            year: value.year.clone(),
            num_seasons: value.number_of_seasons,
            images: value
                .images
                .as_ref()
                .map(Images::from)
                .unwrap_or(Images::default()),
            rating: value.rating.as_ref().map(Rating::from),
        }
    }
}

impl From<&ShowOverview> for media::ShowOverview {
    fn from(value: &ShowOverview) -> Self {
        Self {
            imdb_id: value.imdb_id.clone(),
            tvdb_id: value.tvdb_id.clone(),
            title: value.title.clone(),
            year: value.year.clone(),
            number_of_seasons: value.num_seasons,
            images: MessageField::some(media::Images::from(value.images())),
            rating: value
                .rating
                .as_ref()
                .map(media::Rating::from)
                .map(MessageField::some)
                .unwrap_or_default(),
            special_fields: Default::default(),
        }
    }
}

impl From<&media::MovieDetails> for MovieDetails {
    fn from(value: &media::MovieDetails) -> Self {
        Self {
            title: value.title.clone(),
            imdb_id: value.imdb_id.clone(),
            year: value.year.clone(),
            runtime: value
                .runtime
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_default(),
            genres: value.genres.clone(),
            synopsis: value.synopsis.clone(),
            rating: value.rating.as_ref().map(Rating::from),
            images: value.images.as_ref().map(Images::from).unwrap_or_default(),
            trailer: value.trailer.as_ref().cloned().unwrap_or_default(),
            torrents: value
                .torrents
                .iter()
                .map(|e| {
                    (
                        e.language.clone(),
                        e.torrents
                            .as_ref()
                            .map(|torrents| torrents.into())
                            .unwrap_or_default(),
                    )
                })
                .collect(),
        }
    }
}

impl From<&MovieDetails> for media::MovieDetails {
    fn from(value: &MovieDetails) -> Self {
        Self {
            title: value.title.clone(),
            imdb_id: value.imdb_id.clone(),
            year: value.year.clone(),
            runtime: value.runtime.parse::<u32>().ok(),
            genres: value.genres.clone(),
            synopsis: value.synopsis.clone(),
            rating: value
                .rating
                .as_ref()
                .map(media::Rating::from)
                .map(MessageField::some)
                .unwrap_or_default(),
            images: MessageField::some(media::Images::from(&value.images)),
            trailer: Some(value.trailer.clone()).filter(|e| !e.is_empty()),
            torrents: value
                .torrents
                .iter()
                .map(|(language, torrents)| media::TorrentLanguage {
                    language: language.clone(),
                    torrents: MessageField::some(media::TorrentQuality::from(torrents)),
                    special_fields: Default::default(),
                })
                .collect(),
            special_fields: Default::default(),
        }
    }
}

impl From<&media::ShowDetails> for ShowDetails {
    fn from(value: &media::ShowDetails) -> Self {
        Self {
            imdb_id: value.imdb_id.clone(),
            tvdb_id: value.tvdb_id.clone(),
            title: value.title.clone(),
            year: value.year.clone(),
            num_seasons: value.number_of_seasons,
            images: value.images.as_ref().map(Images::from).unwrap_or_default(),
            rating: value.rating.as_ref().map(Rating::from),
            context_locale: String::new(),
            synopsis: value.synopsis.clone().unwrap_or_default(),
            runtime: value.runtime.as_ref().map(|e| e.to_string()),
            status: value.status.clone().unwrap_or_default(),
            genres: value.genre.clone(),
            episodes: value.episodes.iter().map(Episode::from).collect(),
        }
    }
}

impl From<&ShowDetails> for media::ShowDetails {
    fn from(value: &ShowDetails) -> Self {
        Self {
            imdb_id: value.imdb_id.clone(),
            tvdb_id: value.tvdb_id.clone(),
            title: value.title.clone(),
            year: value.year.clone(),
            number_of_seasons: value.num_seasons,
            images: MessageField::some(media::Images::from(&value.images)),
            rating: value.rating.as_ref().map(media::Rating::from).into(),
            synopsis: Some(value.synopsis.clone()),
            runtime: Some(value.runtime()),
            status: Some(value.status.clone()),
            genre: value.genres.clone(),
            episodes: value.episodes.iter().map(media::Episode::from).collect(),
            special_fields: Default::default(),
        }
    }
}

impl From<&media::Episode> for Episode {
    fn from(value: &media::Episode) -> Self {
        Self {
            season: value.season,
            episode: value.episode,
            first_aired: value.first_aired,
            title: value.title.clone(),
            overview: value.synopsis.clone(),
            tvdb_id: value.tvdb_id.parse::<i32>().unwrap_or_default(),
            tvdb_id_value: value.tvdb_id.clone(),
            thumb: value.thumb.clone(),
            torrents: value
                .torrents
                .as_ref()
                .map(|e| e.into())
                .unwrap_or_default(),
        }
    }
}

impl From<&Episode> for media::Episode {
    fn from(value: &Episode) -> Self {
        Self {
            season: value.season,
            episode: value.episode,
            first_aired: value.first_aired,
            title: value.title.clone(),
            synopsis: value.overview.clone(),
            tvdb_id: value.tvdb_id(),
            thumb: value.thumb.clone(),
            torrents: MessageField::some(media::TorrentQuality::from(&value.torrents)),
            special_fields: Default::default(),
        }
    }
}

impl From<&media::Rating> for Rating {
    fn from(value: &media::Rating) -> Self {
        Self {
            percentage: value.percentage as u16,
            watching: value.watching,
            votes: value.votes,
            loved: value.loved,
            hated: value.hated,
        }
    }
}

impl From<&Rating> for media::Rating {
    fn from(value: &Rating) -> Self {
        Self {
            percentage: value.percentage as u32,
            watching: value.watching,
            votes: value.votes,
            loved: value.loved,
            hated: value.hated,
            special_fields: Default::default(),
        }
    }
}

impl From<&media::Images> for Images {
    fn from(value: &media::Images) -> Self {
        // TODO: update Images to support optional fields as some image urls are empty
        Self {
            poster: value.poster.clone().unwrap_or(String::new()),
            fanart: value.fanart.clone().unwrap_or(String::new()),
            banner: value.banner.clone().unwrap_or(String::new()),
        }
    }
}

impl From<&Images> for media::Images {
    fn from(value: &Images) -> Self {
        Self {
            poster: Some(value.poster.clone()).filter(|v| !v.is_empty()),
            banner: Some(value.banner.clone()).filter(|v| !v.is_empty()),
            fanart: Some(value.fanart.clone()).filter(|v| !v.is_empty()),
            special_fields: Default::default(),
        }
    }
}

impl From<&MediaError> for media::Error {
    fn from(value: &MediaError) -> Self {
        let mut err = Self::new();

        match value {
            MediaError::FavoritesLoadingFailed(e) => {
                err.type_ = media::error::Type::FAVORITES_LOADING_FAILED.into();
                err.favorite_loading_failed =
                    MessageField::some(media::error::FavoritesLoadingFailed {
                        reason: e.clone(),
                        special_fields: Default::default(),
                    })
            }
            MediaError::FavoriteNotFound(e) => {
                err.type_ = media::error::Type::FAVORITE_NOT_FOUND.into();
                err.favorite_not_found = MessageField::some(media::error::FavoriteNotFound {
                    imdb_id: e.clone(),
                    special_fields: Default::default(),
                });
            }
            MediaError::FavoriteAddFailed(imdb_id, reason) => {
                err.type_ = media::error::Type::FAVORITE_ADD_FAILED.into();
                err.favorite_add_failed = MessageField::some(media::error::FavoriteAddFailed {
                    imdb_id: imdb_id.clone(),
                    reason: reason.clone(),
                    special_fields: Default::default(),
                });
            }
            MediaError::WatchedLoadingFailed(reason) => {
                err.type_ = media::error::Type::WATCHED_LOADING_FAILED.into();
                err.watched_loading_failed =
                    MessageField::some(media::error::WatchedLoadingFailed {
                        reason: reason.clone(),
                        special_fields: Default::default(),
                    });
            }
            MediaError::MediaTypeNotSupported(_) => {
                err.type_ = media::error::Type::MEDIA_TYPE_NOT_SUPPORTED.into();
            }
            MediaError::NoAvailableProviders => {
                err.type_ = media::error::Type::NO_AVAILABLE_PROVIDERS.into();
            }
            MediaError::ProviderConnectionFailed => {
                err.type_ = media::error::Type::PROVIDER_CONNECTION_FAILED.into();
            }
            MediaError::ProviderRequestFailed(url, status) => {
                err.type_ = media::error::Type::PROVIDER_REQUEST_FAILED.into();
                err.provider_request_failed =
                    MessageField::some(media::error::ProviderRequestFailed {
                        url: url.clone(),
                        status_code: *status as u32,
                        special_fields: Default::default(),
                    });
            }
            MediaError::ProviderParsingFailed(reason) => {
                err.type_ = media::error::Type::PROVIDER_PARSING_FAILED.into();
                err.provider_parsing_failed =
                    MessageField::some(media::error::ProviderParsingFailed {
                        reason: reason.clone(),
                        special_fields: Default::default(),
                    });
            }
            MediaError::ProviderAlreadyExists(_) => {
                err.type_ = media::error::Type::PROVIDER_ALREADY_EXISTS.into();
            }
            MediaError::ProviderNotFound(e) => {
                err.type_ = media::error::Type::PROVIDER_NOT_FOUND.into();
                err.provider_not_found = MessageField::some(media::error::ProviderNotFound {
                    provider_type: e.clone(),
                    special_fields: Default::default(),
                });
            }
            MediaError::ProviderTimeout => {
                err.type_ = media::error::Type::PROVIDER_TIMEOUT.into();
            }
            MediaError::AutoResumeLoadingFailed(_) => {
                err.type_ = media::error::Type::AUTO_RESUME_LOADING_FAILED.into();
            }
        }

        err
    }
}

impl TryFrom<&media::Item> for Box<dyn MediaOverview> {
    type Error = Error;

    fn try_from(value: &media::Item) -> Result<Self> {
        match value.type_.as_str() {
            media::MovieOverview::NAME => value
                .movie_overview
                .as_ref()
                .map(MovieOverview::from)
                .map(|e| Box::new(e) as Box<dyn MediaOverview>)
                .ok_or(Error::MissingField),
            media::ShowOverview::NAME => value
                .show_overview
                .as_ref()
                .map(ShowOverview::from)
                .map(|e| Box::new(e) as Box<dyn MediaOverview>)
                .ok_or(Error::MissingField),
            media::MovieDetails::NAME => value
                .movie_details
                .as_ref()
                .map(MovieDetails::from)
                .map(|e| Box::new(e) as Box<dyn MediaOverview>)
                .ok_or(Error::MissingField),
            media::ShowDetails::NAME => value
                .show_details
                .as_ref()
                .map(ShowDetails::from)
                .map(|e| Box::new(e) as Box<dyn MediaOverview>)
                .ok_or(Error::MissingField),
            _ => Err(Error::InvalidMessage(value.type_.clone())),
        }
    }
}

impl TryFrom<&media::Item> for Box<dyn MediaIdentifier> {
    type Error = Error;

    fn try_from(value: &media::Item) -> std::result::Result<Self, Self::Error> {
        if value.type_.as_str() == media::Episode::NAME {
            return value
                .episode
                .as_ref()
                .map(Episode::from)
                .map(|e| Box::new(e) as Box<dyn MediaIdentifier>)
                .ok_or(Error::MissingField);
        }

        Ok(Box::<dyn MediaOverview>::try_from(value)? as Box<dyn MediaIdentifier>)
    }
}

impl TryFrom<&Box<dyn MediaIdentifier>> for media::Item {
    type Error = Error;

    fn try_from(value: &Box<dyn MediaIdentifier>) -> Result<Self> {
        if let Some(media) = value.downcast_ref::<MovieOverview>() {
            Ok(media::Item {
                type_: media::MovieOverview::NAME.to_string(),
                movie_overview: MessageField::some(media::MovieOverview::from(media)),
                show_overview: Default::default(),
                movie_details: Default::default(),
                show_details: Default::default(),
                episode: Default::default(),
                special_fields: Default::default(),
            })
        } else if let Some(media) = value.downcast_ref::<ShowOverview>() {
            Ok(media::Item {
                type_: media::ShowOverview::NAME.to_string(),
                movie_overview: Default::default(),
                show_overview: MessageField::some(media::ShowOverview::from(media)),
                movie_details: Default::default(),
                show_details: Default::default(),
                episode: Default::default(),
                special_fields: Default::default(),
            })
        } else if let Some(media) = value.downcast_ref::<MovieDetails>() {
            Ok(media::Item {
                type_: media::MovieDetails::NAME.to_string(),
                movie_overview: Default::default(),
                show_overview: Default::default(),
                movie_details: MessageField::some(media::MovieDetails::from(media)),
                show_details: Default::default(),
                episode: Default::default(),
                special_fields: Default::default(),
            })
        } else if let Some(media) = value.downcast_ref::<ShowDetails>() {
            Ok(media::Item {
                type_: media::ShowDetails::NAME.to_string(),
                movie_overview: Default::default(),
                show_overview: Default::default(),
                movie_details: Default::default(),
                show_details: MessageField::some(media::ShowDetails::from(media)),
                episode: Default::default(),
                special_fields: Default::default(),
            })
        } else if let Some(media) = value.downcast_ref::<Episode>() {
            Ok(media::Item {
                type_: media::Episode::NAME.to_string(),
                movie_overview: Default::default(),
                show_overview: Default::default(),
                movie_details: Default::default(),
                show_details: Default::default(),
                episode: MessageField::some(media::Episode::from(media)),
                special_fields: Default::default(),
            })
        } else {
            Err(Error::MissingField)
        }
    }
}

impl From<&HashMap<String, TorrentInfo>> for media::TorrentQuality {
    fn from(value: &HashMap<String, TorrentInfo>) -> Self {
        Self {
            qualities: value
                .iter()
                .map(|(key, value)| (key.clone(), media::TorrentInfo::from(value)))
                .collect(),
            special_fields: Default::default(),
        }
    }
}

impl From<&media::TorrentQuality> for HashMap<String, TorrentInfo> {
    fn from(value: &media::TorrentQuality) -> Self {
        value
            .qualities
            .iter()
            .map(|(k, info)| (k.clone(), TorrentInfo::from(info)))
            .collect()
    }
}

impl From<&TorrentInfo> for media::TorrentInfo {
    fn from(value: &TorrentInfo) -> Self {
        Self {
            url: value.url().to_string(),
            provider: value.provider().to_string(),
            source: value.source().to_string(),
            title: value.title().clone(),
            quality: value.quality().clone(),
            seeds: *value.seed(),
            peers: *value.peer(),
            size: value.size().map(|e| e.clone()),
            file_size: value.filesize().map(|e| e.clone()),
            file: value.file().map(|e| e.clone()),
            special_fields: Default::default(),
        }
    }
}

impl From<&media::TorrentInfo> for TorrentInfo {
    fn from(value: &media::TorrentInfo) -> Self {
        Self::new(
            value.url.clone(),
            value.provider.clone(),
            value.source.clone(),
            value.title.clone(),
            value.quality.clone(),
            value.seeds.clone(),
            value.peers.clone(),
            value.size.clone(),
            value.file_size.clone(),
            value.file.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::proto::media::media::error;

    #[test]
    fn test_from_category() {
        assert_eq!(
            media::Category::MOVIES,
            media::Category::from(&Category::Movies)
        );
        assert_eq!(
            media::Category::SERIES,
            media::Category::from(&Category::Series)
        );
        assert_eq!(
            media::Category::FAVORITES,
            media::Category::from(&Category::Favorites)
        );
    }

    #[test]
    fn test_from_category_string() {
        assert_eq!("movies".to_string(), String::from(&media::Category::MOVIES));
        assert_eq!("series".to_string(), String::from(&media::Category::SERIES));
        assert_eq!(
            "favorites".to_string(),
            String::from(&media::Category::FAVORITES)
        );
    }

    #[test]
    fn test_torrent_info_from() {
        let info = media::TorrentInfo {
            url: "Url".to_string(),
            provider: "Provider".to_string(),
            source: "Source".to_string(),
            title: "Title".to_string(),
            quality: "1080p".to_string(),
            seeds: 67,
            peers: 3,
            size: Some("100MB".to_string()),
            file_size: Some("10MB".to_string()),
            file: Some("MyTorrentFile.mp4".to_string()),
            special_fields: Default::default(),
        };
        let expected_result = TorrentInfo {
            url: "Url".to_string(),
            provider: "Provider".to_string(),
            source: "Source".to_string(),
            title: "Title".to_string(),
            quality: "1080p".to_string(),
            seed: 67,
            peer: 3,
            size: Some("100MB".to_string()),
            filesize: Some("10MB".to_string()),
            file: Some("MyTorrentFile.mp4".to_string()),
        };

        let result = TorrentInfo::from(&info);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_media_error_from_favorites_loading_failed() {
        let reason = "SomeReason";
        let err = MediaError::FavoritesLoadingFailed(reason.to_string());
        let expected_result = media::Error {
            type_: error::Type::FAVORITES_LOADING_FAILED.into(),
            favorite_loading_failed: MessageField::some(error::FavoritesLoadingFailed {
                reason: reason.to_string(),
                special_fields: Default::default(),
            }),
            favorite_not_found: Default::default(),
            favorite_add_failed: Default::default(),
            watched_loading_failed: Default::default(),
            media_type_not_supported: Default::default(),
            provider_request_failed: Default::default(),
            provider_parsing_failed: Default::default(),
            provider_not_found: Default::default(),
            special_fields: Default::default(),
        };

        let result = media::Error::from(&err);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_media_error_from_favorite_not_found() {
        let id = "tt12000";
        let err = MediaError::FavoriteNotFound(id.to_string());
        let expected_result = media::Error {
            type_: error::Type::FAVORITE_NOT_FOUND.into(),
            favorite_loading_failed: Default::default(),
            favorite_not_found: MessageField::some(error::FavoriteNotFound {
                imdb_id: id.to_string(),
                special_fields: Default::default(),
            }),
            favorite_add_failed: Default::default(),
            watched_loading_failed: Default::default(),
            media_type_not_supported: Default::default(),
            provider_request_failed: Default::default(),
            provider_parsing_failed: Default::default(),
            provider_not_found: Default::default(),
            special_fields: Default::default(),
        };

        let result = media::Error::from(&err);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_media_error_from_favorite_add_failed() {
        let id = "tt12000";
        let reason = "SomeReason";
        let err = MediaError::FavoriteAddFailed(id.to_string(), reason.to_string());
        let expected_result = media::Error {
            type_: error::Type::FAVORITE_ADD_FAILED.into(),
            favorite_loading_failed: Default::default(),
            favorite_not_found: Default::default(),
            favorite_add_failed: MessageField::some(error::FavoriteAddFailed {
                imdb_id: id.to_string(),
                reason: reason.to_string(),
                special_fields: Default::default(),
            }),
            watched_loading_failed: Default::default(),
            media_type_not_supported: Default::default(),
            provider_request_failed: Default::default(),
            provider_parsing_failed: Default::default(),
            provider_not_found: Default::default(),
            special_fields: Default::default(),
        };

        let result = media::Error::from(&err);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_media_error_from_provider_request_failed() {
        let reason = "RequestFailureReason";
        let status = 400;
        let err = MediaError::ProviderRequestFailed(reason.to_string(), status);
        let expected_result = media::Error {
            type_: error::Type::PROVIDER_REQUEST_FAILED.into(),
            favorite_loading_failed: Default::default(),
            favorite_not_found: Default::default(),
            favorite_add_failed: Default::default(),
            watched_loading_failed: Default::default(),
            media_type_not_supported: Default::default(),
            provider_request_failed: MessageField::some(error::ProviderRequestFailed {
                url: reason.to_string(),
                status_code: status as u32,
                special_fields: Default::default(),
            }),
            provider_parsing_failed: Default::default(),
            provider_not_found: Default::default(),
            special_fields: Default::default(),
        };

        let result = media::Error::from(&err);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_media_error_from_provider_parsing_failed() {
        let reason = "ParsingErrorReason";
        let err = MediaError::ProviderParsingFailed(reason.to_string());
        let expected_result = media::Error {
            type_: error::Type::PROVIDER_PARSING_FAILED.into(),
            favorite_loading_failed: Default::default(),
            favorite_not_found: Default::default(),
            favorite_add_failed: Default::default(),
            watched_loading_failed: Default::default(),
            media_type_not_supported: Default::default(),
            provider_request_failed: Default::default(),
            provider_parsing_failed: MessageField::some(error::ProviderParsingFailed {
                reason: reason.to_string(),
                special_fields: Default::default(),
            }),
            provider_not_found: Default::default(),
            special_fields: Default::default(),
        };

        let result = media::Error::from(&err);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_media_error_from_provider_not_found() {
        let provider = "MyProvider";
        let err = MediaError::ProviderNotFound(provider.to_string());
        let expected_result = media::Error {
            type_: error::Type::PROVIDER_NOT_FOUND.into(),
            favorite_loading_failed: Default::default(),
            favorite_not_found: Default::default(),
            favorite_add_failed: Default::default(),
            watched_loading_failed: Default::default(),
            media_type_not_supported: Default::default(),
            provider_request_failed: Default::default(),
            provider_parsing_failed: Default::default(),
            provider_not_found: MessageField::some(error::ProviderNotFound {
                provider_type: provider.to_string(),
                special_fields: Default::default(),
            }),
            special_fields: Default::default(),
        };

        let result = media::Error::from(&err);

        assert_eq!(expected_result, result);
    }
}

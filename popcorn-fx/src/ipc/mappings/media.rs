use crate::ipc::proto::media::media;
use crate::ipc::{Error, Result};
use popcorn_fx_core::core::media::{
    Category, Genre, Images, MediaError, MediaIdentifier, MediaOverview, MovieDetails,
    MovieOverview, Rating, ShowOverview, SortBy, TorrentInfo,
};
use protobuf::{Message, MessageField};

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
            torrents: Default::default(),
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
                    torrents: MessageField::some(media::TorrentQuality {
                        qualities: torrents
                            .iter()
                            .map(|(key, value)| (key.clone(), media::TorrentInfo::from(value)))
                            .collect(),
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                })
                .collect(),
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

impl From<&MediaError> for media::error::Type {
    fn from(value: &MediaError) -> Self {
        match value {
            MediaError::FavoritesLoadingFailed(_) => media::error::Type::FAVORITES_LOADING_FAILED,
            MediaError::FavoriteNotFound(_) => media::error::Type::FAVORITE_NOT_FOUND,
            MediaError::FavoriteAddFailed(_, _) => media::error::Type::FAVORITE_ADD_FAILED,
            MediaError::WatchedLoadingFailed(_) => media::error::Type::WATCHED_LOADING_FAILED,
            MediaError::MediaTypeNotSupported(_) => media::error::Type::MEDIA_TYPE_NOT_SUPPORTED,
            MediaError::NoAvailableProviders => media::error::Type::NO_AVAILABLE_PROVIDERS,
            MediaError::ProviderConnectionFailed => media::error::Type::PROVIDER_CONNECTION_FAILED,
            MediaError::ProviderRequestFailed(_, _) => media::error::Type::PROVIDER_REQUEST_FAILED,
            MediaError::ProviderParsingFailed(_) => media::error::Type::PROVIDER_PARSING_FAILED,
            MediaError::ProviderAlreadyExists(_) => media::error::Type::PROVIDER_ALREADY_EXISTS,
            MediaError::ProviderNotFound(_) => media::error::Type::PROVIDER_NOT_FOUND,
            MediaError::ProviderTimeout => media::error::Type::PROVIDER_TIMEOUT,
            MediaError::AutoResumeLoadingFailed(_) => {
                media::error::Type::AUTO_RESUME_LOADING_FAILED
            }
        }
    }
}

impl From<&MediaError> for media::Error {
    fn from(value: &MediaError) -> Self {
        let mut error = Self::new();
        error.type_ = media::error::Type::from(value).into();

        match value {
            MediaError::FavoritesLoadingFailed(e) => {
                error.favorite_loading_failed =
                    MessageField::some(media::error::FavoritesLoadingFailed {
                        reason: e.clone(),
                        special_fields: Default::default(),
                    })
            }
            MediaError::FavoriteNotFound(e) => {
                error.favorite_not_found = MessageField::some(media::error::FavoriteNotFound {
                    imdb_id: e.clone(),
                    special_fields: Default::default(),
                });
            }
            MediaError::FavoriteAddFailed(_, _) => {}
            MediaError::WatchedLoadingFailed(_) => {}
            MediaError::MediaTypeNotSupported(_) => {}
            MediaError::NoAvailableProviders => {}
            MediaError::ProviderConnectionFailed => {}
            MediaError::ProviderRequestFailed(_, _) => {}
            MediaError::ProviderParsingFailed(_) => {}
            MediaError::ProviderAlreadyExists(_) => {}
            MediaError::ProviderNotFound(e) => {
                error.provider_not_found = MessageField::some(media::error::ProviderNotFound {
                    provider_type: e.clone(),
                    special_fields: Default::default(),
                });
            }
            MediaError::ProviderTimeout => {}
            MediaError::AutoResumeLoadingFailed(_) => {}
        }

        error
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
            _ => Err(Error::InvalidMessage(value.type_.clone())),
        }
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
        } else {
            Err(Error::MissingField)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}

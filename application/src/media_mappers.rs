use std::mem;

use log::trace;

use popcorn_fx_core::{from_c_into_boxed, into_c_owned, MediaItemC, MovieOverviewC, ShowOverviewC, VecFavoritesC};
use popcorn_fx_core::core::media::{MediaIdentifier, MediaOverview, MediaType, MovieOverview, ShowOverview};

pub fn favorites_to_c(favorites: Vec<Box<dyn MediaOverview>>) -> *mut VecFavoritesC {
    trace!("Mapping favorites to VecFavoritesC for {:?}", favorites);
    let mut movies: Vec<MovieOverviewC> = vec![];
    let mut shows: Vec<ShowOverviewC> = vec![];

    for media in favorites.into_iter() {
        if media.media_type() == MediaType::Movie {
            movies.push(MovieOverviewC::from(*media
                .into_any()
                .downcast::<MovieOverview>()
                .expect("expected the media to be a movie overview")))
        } else if media.media_type() == MediaType::Show {
            shows.push(ShowOverviewC::from(*media
                .into_any()
                .downcast::<ShowOverview>()
                .expect("expected the media to be a show overview")));
        }
    }

    into_c_owned(VecFavoritesC::from(movies, shows))
}

pub fn from_media_item(favorite: &MediaItemC) -> Option<Box<dyn MediaIdentifier>> {
    let media: Box<dyn MediaIdentifier>;

    if !favorite.movie_overview.is_null() {
        let boxed = from_c_into_boxed(favorite.movie_overview);
        media = Box::new(boxed.to_struct());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.movie_details.is_null() {
        let boxed = from_c_into_boxed(favorite.movie_details);
        media = Box::new(boxed.to_struct());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.show_overview.is_null() {
        let boxed = from_c_into_boxed(favorite.show_overview);
        media = Box::new(boxed.to_struct());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.show_details.is_null() {
        let boxed = from_c_into_boxed(favorite.show_details);
        media = Box::new(boxed.to_struct());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.episode.is_null() {
        let boxed = from_c_into_boxed(favorite.episode);
        media = Box::new(boxed.to_struct());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else {
        return None;
    }

    Some(media)
}
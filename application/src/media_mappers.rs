use log::trace;

use popcorn_fx_core::{into_c_owned, MovieOverviewC, ShowOverviewC, VecFavoritesC};
use popcorn_fx_core::core::media::{MediaOverview, MediaType, MovieOverview, ShowOverview};

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
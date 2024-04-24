use log::trace;

use popcorn_fx_core::core::media::{MediaOverview, MediaType, MovieOverview, ShowOverview};
use popcorn_fx_core::into_c_owned;

use crate::ffi::{MovieOverviewC, ShowOverviewC, VecFavoritesC};

pub fn favorites_to_c(favorites: Vec<Box<dyn MediaOverview>>) -> *mut VecFavoritesC {
    trace!("Mapping favorites to VecFavoritesC for {:?}", favorites);
    let mut movies: Vec<MovieOverviewC> = vec![];
    let mut shows: Vec<ShowOverviewC> = vec![];

    for media in favorites.into_iter() {
        if media.media_type() == MediaType::Movie {
            movies.push(MovieOverviewC::from(
                *media
                    .into_any()
                    .downcast::<MovieOverview>()
                    .expect("expected the media to be a movie overview"),
            ))
        } else if media.media_type() == MediaType::Show {
            shows.push(ShowOverviewC::from(
                *media
                    .into_any()
                    .downcast::<ShowOverview>()
                    .expect("expected the media to be a show overview"),
            ));
        }
    }

    into_c_owned(VecFavoritesC::from(movies, shows))
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::core::media::Images;

    use super::*;

    #[test]
    fn test_favorites_to_c_movie() {
        let movie = MovieOverview::new(String::new(), "tt54888877".to_string(), String::new());
        let favorites = vec![Box::new(movie) as Box<dyn MediaOverview>];

        let raw = favorites_to_c(favorites);
        let result = unsafe { &*raw };

        assert!(
            !result.movies.is_null(),
            "expected movie array to be filled in"
        );
        assert_eq!(1, result.movies_len)
    }

    #[test]
    fn test_favorites_to_c_show() {
        let show = ShowOverview::new(
            "tt777444111".to_string(),
            String::new(),
            String::new(),
            String::new(),
            1,
            Images::none(),
            None,
        );
        let favorites = vec![Box::new(show) as Box<dyn MediaOverview>];

        let raw = favorites_to_c(favorites);
        let result = unsafe { &*raw };

        assert!(
            !result.shows.is_null(),
            "expected show array to be filled in"
        );
        assert_eq!(1, result.shows_len)
    }
}

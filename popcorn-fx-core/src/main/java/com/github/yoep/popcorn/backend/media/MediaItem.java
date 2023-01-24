package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.media.providers.models.*;
import com.sun.jna.Structure;
import lombok.ToString;

import java.io.Closeable;

@ToString
@Structure.FieldOrder({"movieOverview", "movieDetails", "showOverview", "showDetails"})
public class MediaItem extends Structure implements Closeable {
    public MovieOverview.ByReference movieOverview;
    public MovieDetails.ByReference movieDetails;
    public ShowOverview.ByReference showOverview;
    public ShowDetails.ByReference showDetails;

    public Media getMedia() {
        if (movieOverview != null) {
            return movieOverview;
        }
        if (movieDetails != null) {
            return movieDetails;
        }
        if (showOverview != null) {
            return showOverview;
        }
        return showDetails;
    }

    @Override
    public void close() {
        setAutoSynch(false);
        FxLib.INSTANCE.dispose_media_item(this);
    }

    public static MediaItem from(Media media) {
        var favorite = new MediaItem();

        if (media instanceof MovieDetails.ByReference movie) {
            favorite.movieDetails = movie;
        } else if (media instanceof MovieDetails movie) {
            fromMovieDetails(favorite, movie);
        } else if (media instanceof MovieOverview) {
            favorite.movieOverview = (MovieOverview.ByReference) media;
        } else if (media instanceof ShowDetails.ByReference show) {
            favorite.showDetails = show;
        } else if (media instanceof ShowDetails show) {
            fromShowDetails(favorite, show);
        } else {
            favorite.showOverview = (ShowOverview.ByReference) media;
        }

        return favorite;
    }

    private static void fromShowDetails(MediaItem favorite, ShowDetails show) {
        favorite.showDetails = new ShowDetails.ByReference();
        favorite.showDetails.imdbId = show.imdbId;
        favorite.showDetails.tvdbId = show.tvdbId;
        favorite.showDetails.title = show.title;
        favorite.showDetails.year = show.year;
        favorite.showDetails.numberOfSeasons = show.numberOfSeasons;
        favorite.showDetails.images = show.images;
        favorite.showDetails.rating = show.rating;
        favorite.showDetails.synopsis = show.synopsis;
        favorite.showDetails.runtime = show.runtime;
        favorite.showDetails.status = show.status;
        favorite.showDetails.genresRef = show.genresRef;
        favorite.showDetails.genresLen = show.genresLen;
        favorite.showDetails.genresCap = show.genresCap;
        favorite.showDetails.episodesRef = show.episodesRef;
        favorite.showDetails.episodesLen = show.episodesLen;
        favorite.showDetails.episodesCap = show.episodesCap;
    }

    private static void fromMovieDetails(MediaItem favorite, MovieDetails movie) {
        favorite.movieDetails = new MovieDetails.ByReference();
        favorite.movieDetails.title = movie.title;
        favorite.movieDetails.imdbId = movie.imdbId;
        favorite.movieDetails.year = movie.year;
        favorite.movieDetails.rating = movie.rating;
        favorite.movieDetails.images = movie.images;
        favorite.movieDetails.synopsis = movie.synopsis;
        favorite.movieDetails.runtime = movie.runtime;
        favorite.movieDetails.trailer = movie.trailer;
        favorite.movieDetails.genresRef = movie.genresRef;
        favorite.movieDetails.genresLen = movie.genresLen;
        favorite.movieDetails.genresCap = movie.genresCap;
        favorite.movieDetails.torrentEntry = movie.torrentEntry;
        favorite.movieDetails.torrentLen = movie.torrentLen;
        favorite.movieDetails.torrentCap = movie.torrentCap;
    }
}

package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.media.providers.models.*;
import com.sun.jna.Structure;
import lombok.ToString;

import java.io.Closeable;

@ToString
@Structure.FieldOrder({"movieOverview", "movieDetails", "showOverview", "showDetails", "episode"})
public class MediaItem extends Structure implements Closeable {
    public static class ByReference extends MediaItem implements Structure.ByReference {
    }
    
    public MovieOverview.ByReference movieOverview;
    public MovieDetails.ByReference movieDetails;
    public ShowOverview.ByReference showOverview;
    public ShowDetails.ByReference showDetails;
    public Episode.ByReference episode;

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
        if (showDetails != null) {
            return showDetails;
        }

        return episode;
    }

    @Override
    public void close() {
        setAutoSynch(false);
        FxLib.INSTANCE.dispose_media_item(this);
    }

    public static MediaItem from(Media media) {
        var mediaItem = new MediaItem();

        if (media instanceof MovieDetails.ByReference movie) {
            mediaItem.movieDetails = movie;
        } else if (media instanceof MovieDetails movie) {
            mediaItem.fromMovieDetails(movie);
        } else if (media instanceof MovieOverview.ByReference) {
            mediaItem.movieOverview = (MovieOverview.ByReference) media;
        } else if (media instanceof MovieOverview movie) {
            mediaItem.fromMovieOverview(movie);
        } else if (media instanceof ShowDetails.ByReference show) {
            mediaItem.showDetails = show;
        } else if (media instanceof ShowDetails show) {
            mediaItem.fromShowDetails(show);
        } else if (media instanceof ShowOverview.ByReference show) {
            mediaItem.showOverview = show;
        } else {
            mediaItem.episode = (Episode.ByReference) media;
        }

        return mediaItem;
    }

    private void fromShowDetails(ShowDetails show) {
        this.showDetails = new ShowDetails.ByReference();
        this.showDetails.imdbId = show.imdbId;
        this.showDetails.tvdbId = show.tvdbId;
        this.showDetails.title = show.title;
        this.showDetails.year = show.year;
        this.showDetails.numberOfSeasons = show.numberOfSeasons;
        this.showDetails.images = show.images;
        this.showDetails.rating = show.rating;
        this.showDetails.synopsis = show.synopsis;
        this.showDetails.runtime = show.runtime;
        this.showDetails.status = show.status;
        this.showDetails.genresRef = show.genresRef;
        this.showDetails.genresLen = show.genresLen;
        this.showDetails.genresCap = show.genresCap;
        this.showDetails.episodesRef = show.episodesRef;
        this.showDetails.episodesLen = show.episodesLen;
        this.showDetails.episodesCap = show.episodesCap;
    }

    private void fromMovieDetails(MovieDetails movie) {
        this.movieDetails = new MovieDetails.ByReference();
        this.movieDetails.title = movie.title;
        this.movieDetails.imdbId = movie.imdbId;
        this.movieDetails.year = movie.year;
        this.movieDetails.rating = movie.rating;
        this.movieDetails.images = movie.images;
        this.movieDetails.synopsis = movie.synopsis;
        this.movieDetails.runtime = movie.runtime;
        this.movieDetails.trailer = movie.trailer;
        this.movieDetails.genresRef = movie.genresRef;
        this.movieDetails.genresLen = movie.genresLen;
        this.movieDetails.genresCap = movie.genresCap;
        this.movieDetails.torrentEntry = movie.torrentEntry;
        this.movieDetails.torrentLen = movie.torrentLen;
        this.movieDetails.torrentCap = movie.torrentCap;
    }

    private void fromMovieOverview(MovieOverview movie) {
        this.movieOverview = new MovieOverview.ByReference();
        this.movieOverview.title = movie.title;
        this.movieOverview.imdbId = movie.imdbId;
        this.movieOverview.year = movie.year;
        this.movieOverview.rating = movie.rating;
        this.movieOverview.images = movie.images;
    }
}

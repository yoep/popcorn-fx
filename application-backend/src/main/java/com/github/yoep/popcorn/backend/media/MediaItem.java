package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.media.providers.*;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.io.Closeable;
import java.util.Optional;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"movieOverview", "movieDetails", "showOverview", "showDetails", "episode"})
public class MediaItem extends Structure implements Closeable {
    public static class ByValue extends MediaItem implements Structure.ByValue {
        @Override
        public void close() {
            super.close();
            // TODO: this is cleaned to early, causing the subtitle fetch to fail
//            FxLib.INSTANCE.get().dispose_media_item_value(this);
        }
    }

    public static class ByReference extends MediaItem implements Structure.ByReference {
        @Override
        public void close() {
            super.close();
            // TODO: fix crash on cleanup
            //  FxLib.INSTANCE.get().dispose_media_item(this);
        }
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
        Optional.ofNullable(movieOverview).ifPresent(MovieOverview::close);
        Optional.ofNullable(movieDetails).ifPresent(MovieDetails::close);
        Optional.ofNullable(showOverview).ifPresent(ShowOverview::close);
        Optional.ofNullable(showDetails).ifPresent(ShowDetails::close);
    }

    public MediaItem.ByReference toReference() {
        var media = new MediaItem.ByReference();
        media.showOverview = showOverview;
        media.showDetails = showDetails;
        media.episode = episode;
        media.movieOverview = movieOverview;
        media.movieDetails = movieDetails;
        return media;
    }

    public static MediaItem.ByReference from(Media media) {
        var mediaItem = new MediaItem.ByReference();

        if (media instanceof MovieDetails.ByReference movie) {
            mediaItem.fromMovieDetails(movie);
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
        } else if (media instanceof Episode.ByReference episode) {
            mediaItem.episode = episode;
        } else if (media instanceof Episode episode) {
            mediaItem.fromEpisode(episode);
        } else {
            throw new MediaException(media, "Unsupported media type");
        }

        return mediaItem;
    }

    void fromShowDetails(ShowDetails show) {
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

    void fromMovieDetails(MovieDetails movie) {
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

    void fromMovieOverview(MovieOverview movie) {
        this.movieOverview = new MovieOverview.ByReference();
        this.movieOverview.title = movie.title;
        this.movieOverview.imdbId = movie.imdbId;
        this.movieOverview.year = movie.year;
        this.movieOverview.rating = movie.rating;
        this.movieOverview.images = movie.images;
    }

    void fromEpisode(Episode episode) {
        this.episode = new Episode.ByReference();
        this.episode.season = episode.getSeason();
        this.episode.episode = episode.getEpisode();
        this.episode.firstAired = episode.getFirstAired();
        this.episode.title = episode.getTitle();
        this.episode.synopsis = episode.getSynopsis();
        this.episode.tvdbId = episode.getTvdbId();
        this.episode.thumb = episode.getThumb().orElse(null);
    }
}

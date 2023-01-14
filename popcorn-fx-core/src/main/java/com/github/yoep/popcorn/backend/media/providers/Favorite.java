package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.providers.models.*;
import com.sun.jna.Structure;
import lombok.ToString;

import java.io.Closeable;

@ToString
@Structure.FieldOrder({"movieOverview", "movieDetails", "showOverview", "showDetails"})
public class Favorite extends Structure implements Closeable {
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
    }

    public static Favorite from(Media media) {
        var favorite = new Favorite();

        if (media instanceof MovieDetails.ByReference movie) {
            favorite.movieDetails = movie;
        } else if (media instanceof MovieDetails movie) {
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
        } else if (media instanceof MovieOverview) {
            favorite.movieOverview = (MovieOverview.ByReference) media;
        } else if (media instanceof ShowDetails) {
            favorite.showDetails = (ShowDetails.ByReference) media;
        } else {
            favorite.showOverview = (ShowOverview.ByReference) media;
        }

        return favorite;
    }
}

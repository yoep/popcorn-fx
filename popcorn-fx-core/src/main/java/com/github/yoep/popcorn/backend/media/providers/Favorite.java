package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.favorites.models.Favorable;
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

    public static Favorite from(Favorable media) {
        var favorite = new Favorite();

        if (media instanceof MovieDetails) {
            favorite.movieDetails = (MovieDetails.ByReference) media;
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

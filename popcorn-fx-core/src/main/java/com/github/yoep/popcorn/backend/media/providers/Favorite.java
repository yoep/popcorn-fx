package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.favorites.models.Favorable;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.sun.jna.Structure;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@ToString
@Structure.FieldOrder({"movie", "showOverview", "showDetails"})
public class Favorite extends Structure implements Closeable {
    public Movie.ByReference movie;
    public ShowOverview.ByReference showOverview;
    public ShowDetails.ByReference showDetails;

    public Media getMedia() {
        return Optional.ofNullable(movie)
                .map(e -> (Media) e)
                .orElseGet(() -> Optional.ofNullable(showOverview)
                        .map(e -> (Media) e)
                        .orElse(showDetails));
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    public static Favorite from(Favorable media) {
        var favorite = new Favorite();

        if (media instanceof Movie) {
            favorite.movie = (Movie.ByReference) media;
        } else if (media instanceof ShowOverview) {
            favorite.showOverview = (ShowOverview.ByReference) media;
        } else {
            favorite.showDetails = (ShowDetails.ByReference) media;
        }

        return favorite;
    }
}

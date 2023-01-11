package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.sun.jna.Structure;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@ToString
@Structure.FieldOrder({"movie", "show"})
public class Favorite extends Structure implements Closeable {
    public Movie.ByReference movie;
    public ShowDetails.ByReference show;

    public Media getMedia() {
        return Optional.ofNullable(movie)
                .map(e -> (Media) e)
                .orElse(show);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

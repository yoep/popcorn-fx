package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"movies", "moviesLen", "shows", "showsLen"})
public class MediaSet extends Structure implements Closeable {
    public MovieOverview.ByReference movies;
    public int moviesLen;
    public ShowOverview.ByReference shows;
    public int showsLen;

    public List<MovieOverview> getMovies() {
        return Optional.ofNullable(movies)
                .map(e -> (MovieOverview[]) e.toArray(moviesLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    public List<ShowOverview> getShows() {
        return Optional.ofNullable(shows)
                .map(e -> (ShowOverview[]) e.toArray(showsLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

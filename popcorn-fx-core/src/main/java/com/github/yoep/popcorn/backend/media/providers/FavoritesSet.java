package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.sun.jna.Structure;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;
import java.util.stream.Stream;

@ToString
@Structure.FieldOrder({"movies", "moviesLen", "moviesCap", "shows", "showsLen", "showsCap"})
public class FavoritesSet extends Structure implements Closeable {
    public Movie.ByReference movies;
    public int moviesLen;
    public int moviesCap;
    public ShowOverview.ByReference shows;
    public int showsLen;
    public int showsCap;

    public <T> List<T> getAll() {
        return Stream.concat(getMovies().stream(), getShows().stream())
                .map(e -> (T) e)
                .collect(Collectors.toList());
    }

    public List<Media> getMovies() {
        return Optional.ofNullable(movies)
                .map(e -> e.toArray(moviesLen))
                .map(e -> (Media[]) e)
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    public List<Media> getShows() {
        return Optional.ofNullable(shows)
                .map(e -> e.toArray(showsLen))
                .map(e -> (Media[]) e)
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

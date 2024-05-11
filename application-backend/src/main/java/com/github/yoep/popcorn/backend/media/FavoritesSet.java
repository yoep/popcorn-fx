package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.media.providers.Media;
import com.github.yoep.popcorn.backend.media.providers.MovieOverview;
import com.github.yoep.popcorn.backend.media.providers.ShowOverview;
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
@Structure.FieldOrder({"movies", "moviesLen", "shows", "showsLen"})
public class FavoritesSet extends Structure implements Closeable {
    public MovieOverview.ByReference movies;
    public int moviesLen;
    public ShowOverview.ByReference shows;
    public int showsLen;

    private List<Media> cachedMovies;
    private List<Media> cachedShows;

    public <T extends Media> List<T> getAll() {
        return Stream.concat(cachedMovies.stream(), cachedShows.stream())
                .map(e -> (T) e)
                .collect(Collectors.toList());
    }

    public List<Media> getMovies() {
        return cachedMovies;
    }

    public List<Media> getShows() {
        return cachedShows;
    }

    @Override
    public void read() {
        super.read();
        this.cachedMovies = Optional.ofNullable(movies)
                .map(e -> e.toArray(moviesLen))
                .map(e -> (Media[]) e)
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
        this.cachedShows = Optional.ofNullable(shows)
                .map(e -> e.toArray(showsLen))
                .map(e -> (Media[]) e)
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(movies)
                .map(e -> (MovieOverview[]) e.toArray(moviesLen))
                .stream()
                .flatMap(Arrays::stream)
                .forEach(MovieOverview::close);
        Optional.ofNullable(shows)
                .map(e -> (ShowOverview[]) e.toArray(showsLen))
                .stream()
                .flatMap(Arrays::stream)
                .forEach(ShowOverview::close);
        FxLib.INSTANCE.get().dispose_favorites(this);
    }
}

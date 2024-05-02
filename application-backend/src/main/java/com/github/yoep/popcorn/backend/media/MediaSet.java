package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.media.providers.MovieOverview;
import com.github.yoep.popcorn.backend.media.providers.ShowOverview;
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
    public static class ByValue extends MediaSet implements Structure.ByValue {
        @Override
        public void close() {
            super.close();
            FxLib.INSTANCE.get().dispose_media_items(this);
        }
    }

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
    }
}

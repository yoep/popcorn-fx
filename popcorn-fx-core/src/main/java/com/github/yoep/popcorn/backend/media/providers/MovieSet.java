package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.List;

import static java.util.Arrays.asList;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"movies", "len"})
public class MovieSet extends Structure implements Closeable {
    public MovieOverview.ByReference movies;
    public int len;

    public List<MovieOverview> getMovies() {
        return asList((MovieOverview[]) movies.toArray(len));
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
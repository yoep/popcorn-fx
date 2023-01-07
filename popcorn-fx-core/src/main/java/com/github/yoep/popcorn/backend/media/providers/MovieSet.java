package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.List;

import static java.util.Arrays.asList;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"movies", "len", "cap"})
public class MovieSet extends Structure implements Closeable {
    public Movie.ByReference movies;
    public int len;
    public int cap;

    public List<Movie> getMovies() {
        return asList((Movie[]) movies.toArray(len));
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

package com.github.yoep.popcorn.backend.config.properties;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"name", "genres", "genresLen", "sortBy", "sortByLen"})
public class ProviderProperties extends Structure implements Closeable {
    public static class ByReference extends ProviderProperties implements Structure.ByReference {
    }

    public String name;
    public Pointer genres;
    public int genresLen;
    public Pointer sortBy;
    public int sortByLen;

    private List<String> sortByCache;
    private List<String> genresCache;

    public List<String> getGenres() {
        return genresCache;
    }

    public List<String> getSortBy() {
        return sortByCache;
    }

    @Override
    public void read() {
        super.read();
        sortByCache = Optional.ofNullable(sortBy)
                .map(e -> e.getStringArray(0, sortByLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
        genresCache = Optional.ofNullable(genres)
                .map(e -> e.getStringArray(0, genresLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

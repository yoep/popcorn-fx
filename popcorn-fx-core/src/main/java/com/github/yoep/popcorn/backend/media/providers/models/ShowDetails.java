package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Getter
@ToString
@NoArgsConstructor
@AllArgsConstructor
@JsonIgnoreProperties({"autoAllocate", "stringEncoding", "typeMapper", "fields", "pointer"})
@Structure.FieldOrder({"synopsis", "runtime", "status", "genresRef", "genresLen", "genresCap", "episodesRef", "len", "cap"})
public class ShowDetails extends ShowOverview implements Media, Closeable {
    public static class ByReference extends ShowDetails implements Structure.ByReference {
    }

    public String synopsis;
    public String runtime;
    public String status;
    @JsonIgnore
    public Pointer genresRef;
    @JsonIgnore
    public int genresLen;
    @JsonIgnore
    public int genresCap;
    @JsonIgnore
    public Episode.ByReference episodesRef;
    @JsonIgnore
    public int len;
    @JsonIgnore
    public int cap;

    @JsonIgnore
    private List<Episode> cache;

    public List<Episode> getEpisodes() {
        if (cache == null) {
            cache = Optional.ofNullable(episodesRef)
                    .map(e -> e.toArray(len))
                    .map(e -> (Episode[]) e)
                    .map(Arrays::asList)
                    .orElse(Collections.emptyList());
        }

        return cache;
    }

    @Override
    public String getSynopsis() {
        return synopsis;
    }

    @Override
    public Integer getRuntime() {
        return Integer.parseInt(runtime);
    }

    @Override
    public List<String> getGenres() {
        return Optional.ofNullable(genresRef)
                .map(e -> genresRef.getStringArray(0, genresLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

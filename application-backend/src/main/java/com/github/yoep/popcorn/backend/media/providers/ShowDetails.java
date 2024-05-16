package com.github.yoep.popcorn.backend.media.providers;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Slf4j
@Getter
@ToString
@NoArgsConstructor
@Structure.FieldOrder({"synopsis", "runtime", "status", "genresRef", "genresLen", "genresCap", "episodesRef", "episodesLen", "episodesCap"})
public class ShowDetails extends ShowOverview implements Media, Closeable {
    public static class ByReference extends ShowDetails implements Structure.ByReference {
    }

    public String synopsis;
    public Integer runtime;
    public String status;
    public Pointer genresRef;
    public int genresLen;
    public int genresCap;
    public Episode.ByReference episodesRef;
    public int episodesLen;
    public int episodesCap;

    private List<Episode> cache;

    @Builder
    public ShowDetails(String imdbId, String tvdbId, String title, String year, int numberOfSeasons, Images images, Rating.ByReference rating,
                       String synopsis, Integer runtime, String status, Pointer genresRef, int genresLen, int genresCap, Episode.ByReference episodesRef,
                       int episodesLen, int episodesCap) {
        super(imdbId, tvdbId, title, year, numberOfSeasons, images, rating);
        this.synopsis = synopsis;
        this.runtime = runtime;
        this.status = status;
        this.genresRef = genresRef;
        this.genresLen = genresLen;
        this.genresCap = genresCap;
        this.episodesRef = episodesRef;
        this.episodesLen = episodesLen;
        this.episodesCap = episodesCap;
    }

    public List<Episode> getEpisodes() {
        if (cache == null) {
            cache = Optional.ofNullable(episodesRef)
                    .map(e -> e.toArray(episodesLen))
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
    public List<String> getGenres() {
        return Optional.ofNullable(genresRef)
                .map(e -> genresRef.getStringArray(0, genresLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        super.close();
        Optional.ofNullable(episodesRef)
                .ifPresent(Episode::close);
    }
}

package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.List;
import java.util.Map;

@Getter
@ToString(callSuper = true, exclude = "torrents")
@EqualsAndHashCode(callSuper = true)
@Structure.FieldOrder({"id", "title"})
public class Movie extends AbstractMedia implements Closeable {
    public static class ByReference extends Movie implements Structure.ByReference {
        public ByReference(Movie movie) {
            super(movie.getId(), movie.getTitle());
        }
    }

    private String trailer;
    private Map<String, Map<String, MediaTorrentInfo>> torrents;

    public Movie() {
    }

    public Movie(String id, String title) {
        this.id = id;
        this.title = title;
    }

    @Builder
    public Movie(String id, String imdbId, String title, String year, Integer runtime, List<String> genres, Rating rating, Images images, String synopsis,
                 String trailer, Map<String, Map<String, MediaTorrentInfo>> torrents) {
        super(id, imdbId, title, year, runtime, genres, rating, images, synopsis);
        this.trailer = trailer;
        this.torrents = torrents;
    }

    @Override
    @JsonIgnore
    public MediaType getType() {
        return MediaType.MOVIE;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

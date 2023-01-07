package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@Getter
@ToString(callSuper = true, exclude = "torrents")
@EqualsAndHashCode(callSuper = true)
@Structure.FieldOrder({"id", "title", "imdbId", "year", "runtime", "rating", "images", "synopsis", "trailer", "torrentEntry", "torrentLen", "torrentCap"})
public class Movie extends AbstractMedia implements Closeable {
    public static class ByReference extends Movie implements Structure.ByReference {
    }

    public String trailer;
    public TorrentEntry.ByReference torrentEntry;
    public int torrentLen;
    public int torrentCap;

    private Map<String, Map<String, MediaTorrentInfo>> torrents;

    public Movie() {
    }

    @Builder
    public Movie(String id, String imdbId, String title, String year, Integer runtime, List<String> genres, Rating rating, Images images, String synopsis,
                 String trailer, Map<String, Map<String, MediaTorrentInfo>> torrents) {
        super(id, imdbId, title, year, runtime, genres, toRatingReference(rating), images, synopsis);
        this.trailer = trailer;
        this.torrents = torrents;
    }

    @Override
    @JsonIgnore
    public MediaType getType() {
        return MediaType.MOVIE;
    }

    public Map<String, Map<String, MediaTorrentInfo>> getTorrents() {
        if (torrents == null) {
            torrents = new HashMap<>();
            var entries = (TorrentEntry[]) torrentEntry.toArray(torrentLen);
            for (TorrentEntry entry : entries) {
                var qualities = new HashMap<String, MediaTorrentInfo>();
                torrents.put(entry.language, qualities);
                for (TorrentQuality quality : entry.getQualities()) {
                    qualities.put(quality.getQuality(), quality.getInfo());
                }
            }
        }

        return torrents;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

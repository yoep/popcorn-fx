package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

@Getter
@ToString(callSuper = true, exclude = "torrents")
@EqualsAndHashCode(callSuper = true)
@NoArgsConstructor
@Structure.FieldOrder({"trailer", "torrentEntry", "torrentLen", "torrentCap"})
public class MovieDetails extends MovieOverview implements Closeable {
    public static class ByReference extends MovieDetails implements Structure.ByReference {
    }

    public String trailer;
    public TorrentEntry.ByReference torrentEntry;
    public int torrentLen;
    public int torrentCap;

    private Map<String, Map<String, MediaTorrentInfo>> torrents;

    @Builder
    public MovieDetails(String title, String imdbId, String year, Rating.ByReference rating, Images images, String trailer, Map<String, Map<String,
            MediaTorrentInfo>> torrents) {
        super(title, imdbId, year, rating, images);
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
            Optional.ofNullable(torrentEntry).ifPresent(e -> {
                var entries = (TorrentEntry[]) e.toArray(torrentLen);
                for (TorrentEntry entry : entries) {
                    var qualities = new HashMap<String, MediaTorrentInfo>();
                    torrents.put(entry.language, qualities);
                    for (TorrentQuality quality : entry.getQualities()) {
                        qualities.put(quality.getQuality(), quality.getInfo());
                    }
                }
            });
        }

        return torrents;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

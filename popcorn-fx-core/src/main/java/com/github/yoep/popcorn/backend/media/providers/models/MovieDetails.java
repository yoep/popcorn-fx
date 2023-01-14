package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.util.*;

@Getter
@ToString(callSuper = true, exclude = {"torrents"})
@EqualsAndHashCode(callSuper = true)
@NoArgsConstructor
@Structure.FieldOrder({"synopsis", "runtime", "trailer", "genresRef", "genresLen", "genresCap", "torrentEntry", "torrentLen", "torrentCap"})
public class MovieDetails extends MovieOverview implements Closeable {
    public static class ByReference extends MovieDetails implements Structure.ByReference {
    }

    public String synopsis;
    public Integer runtime;
    public String trailer;
    @JsonIgnore
    public Pointer genresRef;
    @JsonIgnore
    public int genresLen;
    @JsonIgnore
    public int genresCap;
    @JsonIgnore
    public TorrentEntry.ByReference torrentEntry;
    @JsonIgnore
    public int torrentLen;
    @JsonIgnore
    public int torrentCap;

    private Map<String, Map<String, MediaTorrentInfo>> torrents;

    @Builder
    public MovieDetails(String title, String imdbId, String year, Rating.ByReference rating, Images images, String synopsis, Integer runtime,
                        String trailer, Pointer genresRef, int genresLen, int genresCap, TorrentEntry.ByReference torrentEntry, int torrentLen,
                        int torrentCap, Map<String, Map<String, MediaTorrentInfo>> torrents) {
        super(title, imdbId, year, rating, images);
        this.synopsis = synopsis;
        this.runtime = runtime;
        this.trailer = trailer;
        this.genresRef = genresRef;
        this.genresLen = genresLen;
        this.genresCap = genresCap;
        this.torrentEntry = torrentEntry;
        this.torrentLen = torrentLen;
        this.torrentCap = torrentCap;
        this.torrents = torrents;
    }

    @Override
    @JsonIgnore
    public MediaType getType() {
        return MediaType.MOVIE;
    }

    @Override
    public List<String> getGenres() {
        return Optional.ofNullable(genresRef)
                .map(e -> genresRef.getStringArray(0, genresLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
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

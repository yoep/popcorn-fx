package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import javax.validation.constraints.NotNull;
import java.io.Closeable;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.*;

@Data
@ToString(callSuper = true, exclude = "torrents")
@EqualsAndHashCode(callSuper = false)
@JsonIgnoreProperties({"autoAllocate", "stringEncoding", "typeMapper", "fields", "pointer"})
@Structure.FieldOrder({"season", "episode", "firstAired", "title", "synopsis", "tvdbId", "thumb", "torrentRef", "len"})
public class Episode extends Structure implements Comparable<Episode>, Closeable, Media {
    public static class ByReference extends Episode implements Structure.ByReference {
    }

    public int season;
    public int episode;
    public long firstAired;
    public String title;
    public String synopsis;
    public String tvdbId;
    public String thumb;
    public TorrentQuality.ByReference torrentRef;
    public int len;

    private Map<String, MediaTorrentInfo> torrents;

    private Images images;
    // TODO: remove this should not exist for a property
    private List<String> genres;

    public Episode() {
    }

    @Builder
    public Episode(@JsonProperty("tvdb_id") String tvdbId,
                   Map<String, MediaTorrentInfo> torrents,
                   long firstAired,
                   String title,
                   @JsonProperty("overview") String synopsis,
                   Images images,
                   int episode,
                   int season) {
        this.tvdbId = tvdbId;
        this.title = title;
        this.torrents = torrents;
        this.firstAired = firstAired;
        this.episode = episode;
        this.season = season;
        this.synopsis = synopsis;
        this.images = images;
    }

    /**
     * Create a copy of the given episode.
     *
     * @param episode The episode to copy.
     */
    public Episode(@NotNull Episode episode) {
        this.tvdbId = episode.getId();
        this.title = episode.getTitle();
        this.synopsis = episode.getSynopsis();
        this.torrents = episode.getTorrents();
        this.firstAired = episode.getFirstAired();
        this.episode = episode.getEpisode();
        this.season = episode.getSeason();
    }

    //region Properties

    @Override
    public MediaType getType() {
        return MediaType.SHOW;
    }

    //endregion

    //region Getters

    public String getId() {
        return tvdbId;
    }

    public Map<String, MediaTorrentInfo> getTorrents() {
        if (torrents == null) {
            torrents = new HashMap<>();
            var list = Optional.ofNullable(torrentRef)
                    .map(e -> e.toArray(len))
                    .map(e -> (TorrentQuality[]) e)
                    .map(Arrays::asList)
                    .orElse(Collections.emptyList());

            for (var torrent : list) {
                torrents.put(torrent.getQuality(), torrent.getInfo());
            }
        }

        return torrents;
    }

    /**
     * Get the local date time of the air date from this episode.
     *
     * @return Returns the air date as local date time.
     */
    public LocalDateTime getAirDate() {
        return LocalDateTime.ofInstant(Instant.ofEpochSecond(firstAired), ZoneOffset.UTC);
    }

    public Optional<String> getThumb() {
        return Optional.ofNullable(thumb);
    }

    //endregion

    @Override
    public int compareTo(Episode compareTo) {
        // order first by season
        var seasonCompareResult = Integer.compare(season, compareTo.getSeason());

        // if the seasons don't match
        // return the compare result
        if (seasonCompareResult != 0) {
            return seasonCompareResult;
        }

        // order by episode number
        return Integer.compare(episode, compareTo.getEpisode());
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(torrentRef)
                .ifPresent(TorrentQuality::close);
    }

    @Override
    public String getYear() {
        return String.valueOf(getAirDate().getYear());
    }

    @Override
    public Integer getRuntime() {
        return 0;
    }

    @Override
    public List<String> getGenres() {
        return Collections.emptyList();
    }

    @Override
    public Optional<Rating> getRating() {
        return Optional.empty();
    }
}

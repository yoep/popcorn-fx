package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import javax.validation.constraints.NotNull;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.Collections;
import java.util.Map;

@Getter
@ToString(callSuper = true, exclude = "torrents")
@EqualsAndHashCode(callSuper = true)
public class Episode extends AbstractMedia implements Comparable<Episode> {
    /**
     * The available torrents for the episode.
     */
    private final Map<String, MediaTorrentInfo> torrents;
    /**
     * The first air time of the episode
     */
    private final long firstAired;
    /**
     * The episode number
     */
    private final int episode;
    /**
     * The season of the episode
     */
    private final int season;

    @Builder
    public Episode(@JsonProperty("tvdb_id") String tvdbId,
                   Map<String, MediaTorrentInfo> torrents,
                   long firstAired,
                   String title,
                   @JsonProperty("overview") String synopsis,
                   Images images,
                   int episode,
                   int season) {
        super(tvdbId, null, title, null, null, Collections.emptyList(), null, images, synopsis);
        this.torrents = torrents;
        this.firstAired = firstAired;
        this.episode = episode;
        this.season = season;
    }

    /**
     * Create a copy of the given episode.
     *
     * @param episode The episode to copy.
     */
    public Episode(@NotNull Episode episode) {
        super(episode.getId(), null, episode.getTitle(), null, null, Collections.emptyList(), null, episode.getImages(), episode.getSynopsis());
        this.torrents = episode.getTorrents();
        this.firstAired = episode.getFirstAired();
        this.episode = episode.getEpisode();
        this.season = episode.getSeason();
    }

    //region Getters

    @Override
    public String getYear() {
        return String.valueOf(getAirDate().getYear());
    }

    @Override
    @JsonIgnore
    public MediaType getType() {
        return MediaType.SHOW;
    }

    /**
     * Get the local date time of the air date from this episode.
     *
     * @return Returns the air date as local date time.
     */
    public LocalDateTime getAirDate() {
        return LocalDateTime.ofInstant(Instant.ofEpochSecond(firstAired), ZoneOffset.UTC);
    }

    //endregion

    //region Comparable

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

    //endregion
}

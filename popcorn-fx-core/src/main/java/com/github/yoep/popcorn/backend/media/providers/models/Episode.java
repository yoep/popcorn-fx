package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.sun.jna.Structure;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import javax.validation.constraints.NotNull;
import java.io.Closeable;
import java.io.IOException;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Optional;

@Data
@ToString(callSuper = true, exclude = "torrents")
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"id", "title", "season", "episode"})
public class Episode extends Structure implements Comparable<Episode>, Closeable, Media {
    @JsonIgnore
    private final transient BooleanProperty watched = new SimpleBooleanProperty(this, WATCHED_PROPERTY);
    @JsonIgnore
    private final transient BooleanProperty liked = new SimpleBooleanProperty(this, LIKED_PROPERTY);

    public String id;
    public String title;
    public int season;
    public int episode;

    /**
     * The available torrents for the episode.
     */
    private final Map<String, MediaTorrentInfo> torrents;
    /**
     * The first air time of the episode
     */
    private final long firstAired;
    private String synopsis;
    private Images images;
    // TODO: remove this should not exist for a property
    private List<String> genres;

    @Builder
    public Episode(@JsonProperty("tvdb_id") String tvdbId,
                   Map<String, MediaTorrentInfo> torrents,
                   long firstAired,
                   String title,
                   @JsonProperty("overview") String synopsis,
                   Images images,
                   int episode,
                   int season) {
        this.id = tvdbId;
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
        this.id = episode.getId();
        this.title = episode.getTitle();
        this.synopsis = episode.getSynopsis();
        this.torrents = episode.getTorrents();
        this.firstAired = episode.getFirstAired();
        this.episode = episode.getEpisode();
        this.season = episode.getSeason();
    }

    //region Properties

    @Override
    public boolean isWatched() {
        return watched.get();
    }

    @Override
    public BooleanProperty watchedProperty() {
        return watched;
    }

    @Override
    public void setWatched(boolean watched) {
        this.watched.set(watched);
    }

    @Override
    public MediaType getType() {
        return MediaType.SHOW;
    }

    @Override
    public boolean isLiked() {
        return liked.get();
    }

    @Override
    public BooleanProperty likedProperty() {
        return liked;
    }

    @Override
    public void setLiked(boolean liked) {
        this.liked.set(liked);
    }

    //endregion

    //region Getters

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

    @Override
    public void close() throws IOException {
        setAutoSynch(false);
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

    //endregion
}

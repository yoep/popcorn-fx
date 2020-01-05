package com.github.yoep.popcorn.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.github.yoep.popcorn.watched.models.Watchable;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.apache.commons.text.StringEscapeUtils;

import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.Collections;
import java.util.List;
import java.util.Map;

@Data
@EqualsAndHashCode(exclude = {"watched", "show"})
@ToString(exclude = {"watched", "show"})
public class Episode implements Media, Watchable {
    @JsonIgnore
    private BooleanProperty watched = new SimpleBooleanProperty(this, WATCHED_PROPERTY);

    /**
     * The unique video ID of the {@link Episode}.
     * This value is based on the ID of TVDB.
     */
    @JsonProperty("tvdb_id")
    private String id;
    /**
     * The show parent of the episode.
     */
    private Show show;
    /**
     * The available torrents for the episode.
     */
    private Map<String, TorrentInfo> torrents;
    /**
     * The first air time of the episode
     */
    private long firstAired;
    /**
     * The episode's title
     */
    private String title;
    /**
     * The description text of the episode
     */
    @JsonProperty("overview")
    private String synopsis;
    /**
     * The episode number
     */
    private int episode;
    /**
     * The season of the episode
     */
    private int season;

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

    //endregion

    //region Getters

    @Override
    public String getTitle() {
        return StringEscapeUtils.unescapeHtml4(title);
    }

    @Override
    public String getSynopsis() {
        return StringEscapeUtils.unescapeHtml4(synopsis);
    }

    @Override
    public String getYear() {
        return String.valueOf(getAirDate().getYear());
    }

    @Override
    public Integer getRuntime() {
        return show.getRuntime();
    }

    @Override
    public List<String> getGenres() {
        return Collections.emptyList();
    }

    @Override
    public Rating getRating() {
        return null;
    }

    @Override
    public Images getImages() {
        return show.getImages();
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
}

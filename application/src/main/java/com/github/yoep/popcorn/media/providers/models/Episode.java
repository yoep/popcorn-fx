package com.github.yoep.popcorn.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.github.yoep.popcorn.watched.models.Watchable;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import lombok.Data;
import lombok.ToString;
import org.apache.commons.text.StringEscapeUtils;

import java.util.Map;

@Data
@ToString(exclude = {"watched"})
public class Episode implements Watchable {
    @JsonIgnore
    private BooleanProperty watched = new SimpleBooleanProperty(this, WATCHED_PROPERTY);

    /**
     * The TV DB ID.
     */
    private int tvdbId;
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
     * The overview text of the episode
     */
    private String overview;
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

    /**
     * Get the escaped title of the episode.
     *
     * @return Returns the title of the episode.
     */
    public String getTitle() {
        return StringEscapeUtils.unescapeHtml4(title);
    }

    /**
     * Get the escaped overview of the episode.
     *
     * @return Returns the overview of the episode.
     */
    public String getOverview() {
        return StringEscapeUtils.unescapeHtml4(overview);
    }

    //endregion
}

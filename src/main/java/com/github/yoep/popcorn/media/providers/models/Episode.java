package com.github.yoep.popcorn.media.providers.models;

import lombok.Data;
import org.apache.commons.text.StringEscapeUtils;

import java.time.LocalDateTime;
import java.util.Map;

@Data
public class Episode {
    private int tvdbId;
    private Map<String, Torrent> torrents;
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
}

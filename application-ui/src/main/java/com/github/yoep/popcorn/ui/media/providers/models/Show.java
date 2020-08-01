package com.github.yoep.popcorn.ui.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.springframework.util.CollectionUtils;

import java.util.List;

@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
@Data
public class Show extends AbstractMedia {
    /**
     * The unique TVDB ID of the show.
     */
    private String tvdbId;
    /**
     * The number of seasons for the show.
     */
    @JsonProperty("num_seasons")
    private int numberOfSeasons;
    /**
     * The status of the show.
     */
    private String status;
    /**
     * The episodes available for the show.
     */
    private List<Episode> episodes;
    /**
     * The timestamp of the last update for the show.
     */
    private long lastUpdated;

    /**
     * Set the episodes of the show.
     *
     * @param episodes The available episodes for the show.
     */
    public void setEpisodes(List<Episode> episodes) {
        this.episodes = episodes;

        // link the episode to this show
        if (!CollectionUtils.isEmpty(this.episodes))
            this.episodes.forEach(e -> e.setShow(this));
    }

    @Override
    @JsonIgnore
    public MediaType getType() {
        return MediaType.SHOW;
    }
}

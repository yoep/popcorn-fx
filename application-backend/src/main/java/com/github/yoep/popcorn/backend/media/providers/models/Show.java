package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.List;

@Getter
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
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

    @Builder
    public Show(String id, String imdbId, String title, String year, Integer runtime, List<String> genres, Rating rating, Images images, String synopsis,
                String tvdbId, int numberOfSeasons, String status, List<Episode> episodes, long lastUpdated) {
        super(id, imdbId, title, year, runtime, genres, rating, images, synopsis);
        this.tvdbId = tvdbId;
        this.numberOfSeasons = numberOfSeasons;
        this.status = status;
        this.lastUpdated = lastUpdated;

        setEpisodes(episodes);
    }

    /**
     * Set the episodes of the show.
     *
     * @param episodes The available episodes for the show.
     */
    public void setEpisodes(List<Episode> episodes) {
        this.episodes = episodes;
    }

    @Override
    @JsonIgnore
    public MediaType getType() {
        return MediaType.SHOW;
    }
}

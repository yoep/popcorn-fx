package com.github.yoep.popcorn.ui.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.List;
import java.util.Map;

@Getter
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
public class Movie extends AbstractMedia {
    private String trailer;
    private Map<String, Map<String, MediaTorrentInfo>> torrents;

    @Builder
    public Movie(String id, String imdbId, String title, String year, Integer runtime, List<String> genres, Rating rating, Images images, String synopsis,
                 String trailer, Map<String, Map<String, MediaTorrentInfo>> torrents) {
        super(id, imdbId, title, year, runtime, genres, rating, images, synopsis);
        this.trailer = trailer;
        this.torrents = torrents;
    }

    @Override
    @JsonIgnore
    public MediaType getType() {
        return MediaType.MOVIE;
    }
}

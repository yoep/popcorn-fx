package com.github.yoep.popcorn.media.providers.models;

import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;
import org.apache.commons.text.StringEscapeUtils;

import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Optional;

@Data
@NoArgsConstructor
@AllArgsConstructor
public abstract class AbstractMedia implements Media {
    @JsonProperty("_id")
    private String videoId;
    private String imdbId;
    private String title;
    private String year;
    private Integer runtime;
    private List<String> genres;
    private Rating rating;
    private Images images;
    private String synopsis;
    private Map<String, Map<String, Torrent>> torrents;
    private Map<String, String> subtitles;

    @Override
    public String getTitle() {
        return StringEscapeUtils.unescapeHtml4(title);
    }

    @Override
    public String getSynopsis() {
        return StringEscapeUtils.unescapeHtml4(synopsis);
    }

    @Override
    public List<String> getGenres() {
        return Optional.ofNullable(genres)
                .orElse(Collections.emptyList());
    }
}

package com.github.yoep.popcorn.providers.models;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.github.yoep.popcorn.watched.models.AbstractWatchable;
import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.NoArgsConstructor;
import org.apache.commons.text.StringEscapeUtils;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

@EqualsAndHashCode(callSuper = false)
@Data
@NoArgsConstructor
@AllArgsConstructor
public abstract class AbstractMedia extends AbstractWatchable implements Media {
    /**
     * The unique ID of the media item.
     * This ID is most of the time the IMDB ID.
     */
    @JsonProperty("_id")
    private String id;
    /**
     * The IMDB ID of the media.
     */
    private String imdbId;
    /**
     * The title of the media.
     */
    private String title;
    /**
     * The year that the media was published.
     */
    private String year;
    private Integer runtime;
    private List<String> genres;
    private Rating rating;
    private Images images;
    private String synopsis;

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
    public List<String> getGenres() {
        return Optional.ofNullable(genres)
                .orElse(Collections.emptyList());
    }

    //endregion
}
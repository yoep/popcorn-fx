package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import lombok.*;
import org.apache.commons.text.StringEscapeUtils;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Data
@EqualsAndHashCode(exclude = {"watched", "liked"})
@ToString(exclude = {"watched", "liked"})
@NoArgsConstructor
@AllArgsConstructor
public abstract class AbstractMedia implements Media {
    @JsonIgnore
    private final transient BooleanProperty watched = new SimpleBooleanProperty(this, WATCHED_PROPERTY);
    @JsonIgnore
    private final transient BooleanProperty liked = new SimpleBooleanProperty(this, LIKED_PROPERTY);

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

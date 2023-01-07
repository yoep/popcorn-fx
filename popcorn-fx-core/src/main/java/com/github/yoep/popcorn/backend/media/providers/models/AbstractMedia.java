package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.sun.jna.Structure;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import lombok.*;
import org.apache.commons.text.StringEscapeUtils;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Data
@EqualsAndHashCode(exclude = {"watched", "liked"}, callSuper = false)
@ToString(exclude = {"watched", "liked"})
@NoArgsConstructor
@AllArgsConstructor
@JsonIgnoreProperties({"autoAllocate", "stringEncoding", "typeMapper", "fields", "pointer"})
public abstract class AbstractMedia extends Structure implements Media {
    @JsonIgnore
    private final transient BooleanProperty watched = new SimpleBooleanProperty(this, WATCHED_PROPERTY);
    @JsonIgnore
    private final transient BooleanProperty liked = new SimpleBooleanProperty(this, LIKED_PROPERTY);

    /**
     * The unique ID of the media item.
     * This ID is most of the time the IMDB ID.
     */
    @JsonProperty("_id")
    public String id;
    /**
     * The IMDB ID of the media.
     */
    public String imdbId;
    /**
     * The title of the media.
     */
    public String title;
    /**
     * The year that the media was published.
     */
    public String year;
    public Integer runtime;
    private List<String> genres;
    public Rating.ByReference rating;
    public Images images;
    public String synopsis;

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

    public Optional<Rating> getRating() {
        return Optional.ofNullable(rating);
    }

    @Override
    public List<String> getGenres() {
        return Optional.ofNullable(genres)
                .orElse(Collections.emptyList());
    }

    //endregion

    protected static Rating.ByReference toRatingReference(Rating rating) {
        if (rating == null) {
            return null;
        }

        var reference = new Rating.ByReference();
        reference.percentage = rating.percentage;
        reference.watching = rating.watching;
        reference.votes = rating.votes;
        reference.loved = rating.loved;
        reference.hated = rating.hated;
        return reference;
    }
}

package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.sun.jna.Structure;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import lombok.*;

import java.io.Closeable;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Data
@ToString(exclude = {"watched", "liked"})
@EqualsAndHashCode(callSuper = false)
@NoArgsConstructor
@AllArgsConstructor
@JsonIgnoreProperties({"autoAllocate", "stringEncoding", "typeMapper", "fields", "pointer"})
@Structure.FieldOrder({"title", "imdbId", "year", "rating", "images"})
public class MovieOverview extends Structure implements Media, Closeable {
    public static class ByReference extends MovieOverview implements Structure.ByReference {
    }

    @JsonIgnore
    private final transient BooleanProperty watched = new SimpleBooleanProperty(this, WATCHED_PROPERTY);
    @JsonIgnore
    private final transient BooleanProperty liked = new SimpleBooleanProperty(this, LIKED_PROPERTY);

    public String title;
    public String imdbId;
    public String year;
    public Rating.ByReference rating;
    public Images images;

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
    public MediaType getType() {
        return MediaType.SHOW;
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

    @Override
    public String getId() {
        return imdbId;
    }

    @Override
    public String getSynopsis() {
        return "";
    }

    @Override
    public Integer getRuntime() {
        return 0;
    }

    @Override
    @JsonIgnore
    public List<String> getGenres() {
        return Collections.emptyList();
    }

    public Optional<Rating> getRating() {
        return Optional.ofNullable(rating);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

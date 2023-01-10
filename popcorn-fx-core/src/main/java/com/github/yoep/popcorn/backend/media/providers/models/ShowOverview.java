package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.sun.jna.Structure;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.ToString;

import java.io.Closeable;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@Getter
@ToString(exclude = {"watched", "liked"})
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"id", "imdbId", "tvdbId", "title", "year", "slug", "numberOfSeasons", "images", "rating"})
public class ShowOverview extends Structure implements Media, Closeable {
    public static class ByReference extends ShowOverview implements Structure.ByReference {
    }

    @JsonIgnore
    private final transient BooleanProperty watched = new SimpleBooleanProperty(this, WATCHED_PROPERTY);
    @JsonIgnore
    private final transient BooleanProperty liked = new SimpleBooleanProperty(this, LIKED_PROPERTY);

    public String id;
    public String imdbId;
    public String tvdbId;
    public String title;
    public String year;
    public String slug;
    public int numberOfSeasons;
    public Images images;
    public Rating.ByReference rating;

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

    public String getId() {
        return Optional.ofNullable(id)
                .orElse(imdbId);
    }

    public Optional<Rating> getRating() {
        return Optional.ofNullable(rating);
    }

    @Override
    public String getSynopsis() {
        return "";
    }

    @Override
    public Integer getRuntime() {
        return 0;
    }

    public int getSeasons() {
        return numberOfSeasons;
    }

    @Override
    public List<String> getGenres() {
        return new ArrayList<>();
    }

    public void setGenres(List<String> genres) {
        // no-op
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

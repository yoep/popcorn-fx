package com.github.yoep.popcorn.backend.media.filters.model;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.github.yoep.popcorn.backend.media.providers.models.MediaType;
import com.github.yoep.popcorn.backend.media.watched.models.Watchable;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import lombok.AllArgsConstructor;
import lombok.Data;

@Data
@AllArgsConstructor
public class Season implements Watchable {
    @JsonIgnore
    private final BooleanProperty watched = new SimpleBooleanProperty(this, WATCHED_PROPERTY);

    private final int season;
    private final String text;

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

    //endregion

    //region Getters

    @Override
    public String getId() {
        return null;
    }

    @Override
    @JsonIgnore
    public MediaType getType() {
        return MediaType.SHOW;
    }

    //endregion

    @Override
    public String toString() {
        return text;
    }
}

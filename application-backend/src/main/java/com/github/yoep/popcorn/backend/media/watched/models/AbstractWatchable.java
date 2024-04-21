package com.github.yoep.popcorn.backend.media.watched.models;

import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;

/**
 * Abstract implementation of {@link Watchable}.
 */
public abstract class AbstractWatchable implements Watchable {
    private final BooleanProperty watched = new SimpleBooleanProperty(this, WATCHED_PROPERTY);

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
}

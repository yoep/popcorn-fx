package com.github.yoep.popcorn.watched.models;

import javafx.beans.property.BooleanProperty;

public interface Watchable {
    /**
     * The watchable property name.
     */
    String WATCHED_PROPERTY = "watched";

    /**
     * Check if this watchable has been watched.
     *
     * @return Returns true if this watchable has been watched, else false.
     */
    boolean isWatched();

    /**
     * Get the watched property of this watchable.
     *
     * @return Returns the watched property.
     */
    BooleanProperty watchedProperty();

    /**
     * Set if the watchable has been watched.
     *
     * @param watched The watched value.
     */
    void setWatched(boolean watched);
}
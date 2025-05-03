package com.github.yoep.popcorn.backend.media.watched.models;

import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.providers.MediaType;
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

    /**
     * Get the unique ID of the watchable.
     * This is most of the time the IMDB ID or TVDB ID from the {@link Media}.
     *
     * @return The unique ID of the watchable.
     */
    String getId();

    /**
     * Get the media type of the watchable item.
     *
     * @return Returns the media type.
     */
    MediaType getType();
}

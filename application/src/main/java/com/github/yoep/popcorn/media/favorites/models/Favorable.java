package com.github.yoep.popcorn.media.favorites.models;

import com.github.yoep.popcorn.media.providers.models.MediaType;
import javafx.beans.property.BooleanProperty;

public interface Favorable {
    /**
     * The liked/favorable property name.
     */
    String LIKED_PROPERTY = "liked";

    /**
     * Check if this {@link Favorable} has been liked.
     *
     * @return Returns true this favorable is liked, else false.
     */
    boolean isLiked();

    /**
     * Get the liked property of this {@link Favorable}.
     *
     * @return Returns the liked property.
     */
    BooleanProperty likedProperty();

    /**
     * Set the new liked value of this {@link Favorable}.
     *
     * @param liked The liked value.
     */
    void setLiked(boolean liked);

    /**
     * Get the unique ID of the {@link Favorable}.
     * This is most of the time the IMDB ID or TVDB ID from the {@link com.github.yoep.popcorn.media.providers.models.Media}.
     *
     * @return The unique ID of the favorable.
     */
    String getId();

    /**
     * Get the media type of the {@link Favorable} item.
     *
     * @return Returns the media type.
     */
    MediaType getType();
}

package com.github.yoep.popcorn.view.controllers.components;

import com.github.yoep.popcorn.media.providers.models.Media;

/**
 * Media card item listener for the overlay media card.
 */
public interface OverlayItemListener extends SimpleItemListener {
    /**
     * Invoked when the media favorite/like is being changed.
     *
     * @param media    The media that is being changed.
     * @param newValue The new favorite value of the media.
     */
    void onFavoriteChanged(Media media, boolean newValue);

    /**
     * Invoked when the media watched/viewed is being changed.
     *
     * @param media    The media that is being changed.
     * @param newValue The new value of the watched state for the media.
     */
    void onWatchedChanged(Media media, boolean newValue);
}

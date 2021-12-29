package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.ui.view.controllers.common.SimpleItemListener;

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

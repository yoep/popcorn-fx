package com.github.yoep.popcorn.view.controllers.common;

import com.github.yoep.popcorn.media.providers.models.Media;

/**
 * Media card listener for the simple media card.
 */
public interface SimpleItemListener {
    /**
     * Invoked when the poster item has been clicked.
     *
     * @param media The media of the poster.
     */
    void onClicked(Media media);
}

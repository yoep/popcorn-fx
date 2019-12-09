package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.media.providers.models.Media;

/**
 * Item poster listener.
 */
public interface ItemListener {
    /**
     * Invoked when the poster item has been clicked.
     *
     * @param media The media of the poster.
     */
    void onClicked(Media media);
}

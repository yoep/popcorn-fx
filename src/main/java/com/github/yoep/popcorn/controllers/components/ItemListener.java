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

    /**
     * Invoked when the media favorite/like is being changed.
     *
     * @param media    The media that is being added.
     * @param newValue The new favorite value of the media.
     */
    void onFavoriteChanged(Media media, boolean newValue);
}

package com.github.yoep.popcorn.ui.view.listeners;

public interface DetailsComponentListener {
    /**
     * Invoked when the watched state is changed of the media item.
     *
     * @param imdbId   The IMDB id of the item for which the state changed.
     * @param newState The new watched state.
     */
    void onWatchChanged(String imdbId, boolean newState);

    /**
     * Invoked when the liked state is changed of the media item.
     *
     * @param imdbId
     * @param newState The new liked state.
     */
    void onLikedChanged(String imdbId, boolean newState);
}

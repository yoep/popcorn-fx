package com.github.yoep.popcorn.ui.view.listeners;

public interface DetailsComponentListener {
    /**
     * Invoked when the watched state is changed of the media item.
     *
     * @param newState The new watched state.
     */
    void onWatchChanged(boolean newState);

    /**
     * Invoked when the liked state is changed of the media item.
     *
     * @param newState The new liked state.
     */
    void onLikedChanged(boolean newState);
}
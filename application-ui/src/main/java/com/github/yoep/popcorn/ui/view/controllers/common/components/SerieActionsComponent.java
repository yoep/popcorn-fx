package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;

public interface SerieActionsComponent {
    /**
     * Invoked when the episode is changed for the current series being displayed.
     *
     * @param media   The details of the series being shown.
     * @param episode The newly selected episode.
     */
    void episodeChanged(ShowDetails media, Episode episode);

    /**
     * Sets an event handler to be invoked when the "Watch Now" button is clicked.
     *
     * @param eventHandler The event handler to be executed.
     */
    void setOnWatchNowClicked(Runnable eventHandler);
}


package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;

public interface SerieActionsComponent {
    /**
     * Invoked when the episode is changed of the current serie that is being shown.
     *
     * @param media   The media being displayed.
     * @param episode The currently selected episode.
     */
    void episodeChanged(ShowDetails media, Episode episode);

    void setOnWatchNowClicked(Runnable eventHandler);
}

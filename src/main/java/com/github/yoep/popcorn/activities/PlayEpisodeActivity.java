package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.media.providers.models.Episode;

public interface PlayEpisodeActivity extends PlayVideoActivity {
    /**
     * Get the episode that needs to be played.
     *
     * @return Returns the episode that needs to be played.
     */
    Episode getEpisode();
}

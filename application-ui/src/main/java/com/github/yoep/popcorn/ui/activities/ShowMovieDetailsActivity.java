package com.github.yoep.popcorn.ui.activities;

import com.github.yoep.popcorn.ui.media.providers.models.Movie;

public interface ShowMovieDetailsActivity extends ShowDetailsActivity {
    /**
     * Get the media to show the details of.
     *
     * @return Returns the media to show the details of.
     */
    Movie getMedia();
}

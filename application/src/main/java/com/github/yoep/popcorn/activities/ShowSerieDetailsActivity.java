package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.providers.models.Show;

public interface ShowSerieDetailsActivity extends ShowDetailsActivity {
    /**
     * Get the media to show the details of.
     *
     * @return Returns the media to show the details of.
     */
    Show getMedia();
}

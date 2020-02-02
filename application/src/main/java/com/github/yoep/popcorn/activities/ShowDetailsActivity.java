package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.media.providers.models.Media;

/**
 * Invoked when the details of a media item should be shown.
 */
public interface ShowDetailsActivity extends Activity {
    /**
     * Get the media to show the details of.
     *
     * @return Returns the media to show the details of.
     */
    Media getMedia();
}

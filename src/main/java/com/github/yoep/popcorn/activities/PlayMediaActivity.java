package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.media.providers.models.Media;

public interface PlayMediaActivity extends Activity {
    /**
     * Get the media that needs to be played.
     *
     * @return Returns the media that needs to be played.
     */
    Media getMedia();
}

package com.github.yoep.popcorn.activities;

public interface PlayVideoActivity extends PlayMediaActivity {
    /**
     * Get the url to play the media of.
     *
     * @return Return the media url.
     */
    String getUrl();
}

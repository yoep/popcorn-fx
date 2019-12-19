package com.github.yoep.popcorn.activities;

import java.util.Optional;

public interface PlayVideoActivity extends PlayMediaActivity {
    /**
     * Get the url to play the media of.
     *
     * @return Return the media url.
     */
    String getUrl();

    /**
     * Get the video quality of the media.
     *
     * @return Returns the quality if known for the media, else {@link Optional#empty()}.
     */
    Optional<String> getQuality();
}

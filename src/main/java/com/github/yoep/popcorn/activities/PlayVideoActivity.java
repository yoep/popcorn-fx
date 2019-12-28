package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.subtitle.models.Subtitle;

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

    /**
     * Get the subtitle that needs to be added to the playback of the video.
     *
     * @return Returns the subtitle for the playback if present, else {@link Optional#empty()}.
     */
    Optional<Subtitle> getSubtitle();
}

package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.providers.models.Media;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;

import java.util.Optional;

public interface PlayMediaActivity extends PlayVideoActivity {
    /**
     * Get the media that needs to be played.
     *
     * @return Returns the media that needs to be played.
     */
    Media getMedia();

    /**
     * Get the video quality of the media.
     *
     * @return Returns the quality of the media.
     */
    String getQuality();

    /**
     * Get the subtitle that needs to be added to the playback of the media.
     *
     * @return Returns the subtitle for the playback if present, else {@link Optional#empty()}.
     */
    Optional<SubtitleInfo> getSubtitle();
}

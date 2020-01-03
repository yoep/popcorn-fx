package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.media.providers.models.Media;

import java.util.Optional;

public interface ClosePlayerActivity extends Activity {
    /**
     * The unknown value for the {@link #getTime()} and {@link #getDuration()}.
     */
    long UNKNOWN = -1;

    /**
     * Get the media that was being played.
     *
     * @return Returns the media that was being played.
     */
    Media getMedia();

    /**
     * Get the video quality of the media that was being played.
     *
     * @return Returns the quality if known for the media, else {@link Optional#empty()}.
     */
    Optional<String> getQuality();

    /**
     * Get the last known time of the video player state.
     *
     * @return Returns the last time that was known by the video player, else {@link #UNKNOWN} if missing.
     */
    long getTime();

    /**
     * Get the duration of the video that was being played.
     *
     * @return Returns the length of the video, else {@link #UNKNOWN} if missing.
     */
    long getDuration();
}

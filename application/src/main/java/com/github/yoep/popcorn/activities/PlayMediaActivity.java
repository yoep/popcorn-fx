package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;

import java.util.Optional;

public interface PlayMediaActivity extends Activity {
    /**
     * Get the media that needs to be played.
     *
     * @return Returns the media that needs to be played.
     */
    Media getMedia();

    /**
     * Get the subtitle that needs to be added to the playback of the media.
     *
     * @return Returns the subtitle for the playback if present, else {@link Optional#empty()}.
     */
    Optional<SubtitleInfo> getSubtitle();
}

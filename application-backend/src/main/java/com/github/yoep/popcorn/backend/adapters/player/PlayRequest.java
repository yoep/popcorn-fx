package com.github.yoep.popcorn.backend.adapters.player;

import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;

import javax.validation.constraints.NotNull;
import java.util.Optional;

/**
 * The {@link PlayRequest} contains the information to start a new media playback item in the {@link Player}.
 */
public interface PlayRequest {
    /**
     * Get the playback url.
     *
     * @return Returns the playback url.
     */
    @NotNull
    String getUrl();

    /**
     * The title of the video playback.
     *
     * @return Returns the title of the playback if known, else {@link Optional#empty()}.
     */
    Optional<String> getTitle();

    /**
     * The subtitle that needs to be added to the playback of the video.
     *
     * @return Returns the subtitle if one was selected, else {@link Optional#empty()}.
     */
    Optional<SubtitleInfo> getSubtitle();

    /**
     * Get the thumbnail url if one is present for the video.
     *
     * @return Returns the thumb of the video if available, else {@link Optional#empty()}.
     */
    Optional<String> getThumbnail();

    /**
     * The quality of the video playback.
     *
     * @return Returns the video playback quality if known, else {@link Optional#empty()}.
     */
    Optional<String> getQuality();

    /**
     * The auto resume timestamp of known for the video playback.
     * This timestamp is based on the last time the playback occurred.
     *
     * @return Returns the video playback last timestamp if known, else {@link Optional#empty()}.
     */
    Optional<Long> getAutoResumeTimestamp();

    /**
     * Check if the subtitles are enabled for this {@link PlayRequest}.
     *
     * @return Returns true if subtitles should be enabled for the request, else false.
     */
    boolean isSubtitlesEnabled();
}

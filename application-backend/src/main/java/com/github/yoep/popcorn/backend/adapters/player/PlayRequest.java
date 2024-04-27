package com.github.yoep.popcorn.backend.adapters.player;

import com.github.yoep.popcorn.backend.lib.Handle;

import java.util.Optional;

/**
 * The {@link PlayRequest} interface represents a request to start playback of a media item in the {@link Player}.
 * It contains essential information such as the playback URL, title, optional thumbnail and background URLs,
 * playback quality, auto-resume timestamp, subtitles status, and optional stream handle.
 */
public interface PlayRequest {
    /**
     * Get the playback URL.
     *
     * @return The playback URL.
     */
    
    String getUrl();

    /**
     * Get the title of the media item.
     *
     * @return The title of the media item.
     */
    String getTitle();

    Optional<String> getCaption();

    /**
     * Get the thumbnail URL for the media item if available.
     *
     * @return An {@link Optional} containing the thumbnail URL if available, otherwise {@link Optional#empty()}.
     */
    Optional<String> getThumbnail();

    /**
     * Get the background URL for the media item if available.
     *
     * @return An {@link Optional} containing the background URL if available, otherwise {@link Optional#empty()}.
     */
    Optional<String> getBackground();

    /**
     * Get the playback quality of the media item if known.
     *
     * @return An {@link Optional} containing the playback quality if known, otherwise {@link Optional#empty()}.
     */
    Optional<String> getQuality();

    /**
     * Get the auto-resume timestamp for the media item if known.
     * This timestamp is based on the last time the playback occurred.
     *
     * @return An {@link Optional} containing the auto-resume timestamp if known, otherwise {@link Optional#empty()}.
     */
    Optional<Long> getAutoResumeTimestamp();

    /**
     * Check if subtitles are enabled for this media item.
     *
     * @return {@code true} if subtitles should be enabled for the request, otherwise {@code false}.
     */
    boolean isSubtitlesEnabled();

    /**
     * Get the handle for the media stream if available.
     *
     * @return An {@link Optional} containing the handle for the media stream if available, otherwise {@link Optional#empty()}.
     */
    Optional<Handle> getStreamHandle();
}
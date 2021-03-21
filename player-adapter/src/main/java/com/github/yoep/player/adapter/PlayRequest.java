package com.github.yoep.player.adapter;

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
    String getUrl();

    /**
     * Get the title of the video playback.
     *
     * @return Returns the title of the playback if known, else {@link Optional#empty()}.
     */
    Optional<String> getTitle();
}

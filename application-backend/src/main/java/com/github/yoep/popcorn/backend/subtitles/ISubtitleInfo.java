package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;

public interface ISubtitleInfo {
    /**
     * Get the imdb ID of the subtitle.
     *
     * @return Returns the imdb ID.
     */
    String getImdbId();

    /**
     * Get the language of the subtitle.
     *
     * @return Returns the language of the subtitle.
     */
    Subtitle.Language getLanguage();

    /**
     * Check if this subtitle is the none/disabled type.
     *
     * @return Returns true if the subtitle type is none.
     */
    boolean isNone();

    /**
     * Check if this subtitle is the custom type.
     *
     * @return Returns true if the subtitle type is custom.
     */
    boolean isCustom();

    /**
     * Get the flag resource for this subtitle.
     * The flag resource should exist as the "unknown"/"not supported" languages are already filtered by the {@link com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Language}.
     *
     * @return Returns the flag class path resource.
     */
    String getFlagResource();
}

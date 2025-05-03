package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;

import java.util.List;
import java.util.Optional;

public interface ISubtitle {
    /**
     * Get the filepath of the subtitle.
     *
     * @return Returns the absolute filepath of the subtitle file location.
     */
    String getFilePath();

    /**
     * Get the subtitle cues.
     *
     * @return Returns the cues of this subtitle.
     */
    List<Subtitle.Cue> cues();

    /**
     * Check if this subtitle is the special "none" subtitle.
     *
     * @return Returns true if this subtitle is the "none" subtitle, else false.
     */
    boolean isNone();

    /**
     * Get the subtitle info of this subtitle.
     *
     * @return Returns the subtitle info if present, else {@link Optional#empty()}.
     */
    Optional<Subtitle.Info> getSubtitleInfo();
}

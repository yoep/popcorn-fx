package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.subtitles.Subtitle;

public interface SubtitleListener {
    /**
     * Invoked when a new subtitle becomes active.
     */
    void onSubtitleChanged(Subtitle newSubtitle);

    /**
     * Invoked when the subtitle is being disabled.
     */
    void onSubtitleDisabled();
}

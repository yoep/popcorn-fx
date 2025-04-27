package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.subtitles.ISubtitle;

public interface SubtitleListener {
    /**
     * Invoked when a new subtitle becomes active.
     */
    void onSubtitleChanged(ISubtitle newSubtitle);

    /**
     * Invoked when the subtitle is being disabled.
     */
    void onSubtitleDisabled();
}

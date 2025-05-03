package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.subtitles.ISubtitle;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;

import java.util.List;

public interface SubtitleListener {
    /**
     * Invoked when a new subtitle becomes active.
     */
    void onSubtitleChanged(ISubtitle newSubtitle);

    /**
     * Invoked when the subtitle is being disabled.
     */
    void onSubtitleDisabled();

    /**
     * Invoked when the available subtitles have been changed.
     *
     * @param subtitles The list of available subtitles.
     */
    void onAvailableSubtitlesChanged(List<ISubtitleInfo> subtitles);
}

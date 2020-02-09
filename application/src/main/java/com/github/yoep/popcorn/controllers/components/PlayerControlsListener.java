package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.subtitles.models.SubtitleInfo;

public interface PlayerControlsListener {
    /**
     * Invoked when the subtitle is being changed.
     *
     * @param subtitle The new subtitle.
     */
    void onSubtitleChanged(SubtitleInfo subtitle);

    /**
     * Invoked when the subtitle font size is being changed.
     *
     * @param pixelChange The font size change of the subtitle.
     */
    void onSubtitleSizeChanged(int pixelChange);

    /**
     * Invoked when the time is being changed.
     *
     * @param time The new video time.
     */
    void onTimeChanged(long time);

    /**
     * Invoked when the play/pause button has been clicked.
     */
    void onPlayPauseClicked();
}

package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;

import javax.validation.constraints.NotNull;

public interface PlayerHeaderListener {
    void onTitleChanged(String title);

    void onCaptionChanged(String caption);

    void onQualityChanged(String quality);

    /**
     * Invoked when the streaming state of the video player has changed.
     *
     * @param isStreaming The indication if the video is being streamed ot not.
     */
    void onStreamStateChanged(boolean isStreaming);

    /**
     * Invoked when the streaming download progress has changed.
     *
     * @param progress The last known progress update.
     */
    void onDownloadStatusChanged(@NotNull DownloadStatus progress);
}

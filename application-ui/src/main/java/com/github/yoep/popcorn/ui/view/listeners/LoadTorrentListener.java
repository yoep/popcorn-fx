package com.github.yoep.popcorn.ui.view.listeners;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.media.providers.models.Media;

public interface LoadTorrentListener {

    /**
     * Invoked when the loading state is changed.
     *
     * @param newState The new loading state.
     */
    void onStateChanged(State newState);

    /**
     * Invoked when the media of the loading torrent is changed.
     *
     * @param media The new media item of the loading torrent, can be null if no media item is present.
     */
    void onMediaChanged(Media media);

    /**
     * Invoked when the download status of the torrent is changed.
     *
     * @param status The new status of the download progress.
     */
    void onDownloadStatusChanged(DownloadStatus status);

    enum State {
        STARTING,
        INITIALIZING,
        RETRIEVING_SUBTITLES,
        DOWNLOADING_SUBTITLE,
        CONNECTING,
        DOWNLOADING,
        READY,
        ERROR
    }
}

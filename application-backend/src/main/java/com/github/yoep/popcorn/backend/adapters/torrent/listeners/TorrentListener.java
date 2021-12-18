package com.github.yoep.popcorn.backend.adapters.torrent.listeners;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;

/**
 * A torrent listener which listens on events of a {@link com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent}.
 */
public interface TorrentListener {
    /**
     * Invoked when the torrent state changed.
     *
     * @param oldState The old torrent state.
     * @param newState The new torrent state.
     */
    void onStateChanged(TorrentState oldState, TorrentState newState);

    /**
     * Invoked when the torrent encounters an error.
     *
     * @param error The error that occurred.
     */
    void onError(TorrentException error);

    /**
     * Invoked when the torrent download progress changes.
     *
     * @param status The new download status information.
     */
    void onDownloadProgress(DownloadStatus status);

    /**
     * Invoked when a piece of the torrent has been finished.
     *
     * @param pieceIndex The piece index that has been finished.
     */
    void onPieceFinished(int pieceIndex);
}

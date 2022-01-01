package com.github.yoep.popcorn.backend.adapters.torrent.listeners;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;

/**
 * Abstract implementation which already implements all methods.
 * This allows for a more clean implementation of the {@link TorrentListener} when not all invocations are needed.
 */
public abstract class AbstractTorrentListener implements TorrentListener {
    @Override
    public void onStateChanged(TorrentState oldState, TorrentState newState) {
        //no-op
    }

    @Override
    public void onError(TorrentException error) {
        //no-op
    }

    @Override
    public void onDownloadStatus(DownloadStatus status) {
        //no-op
    }

    @Override
    public void onPieceFinished(int pieceIndex) {
        //no-op
    }
}

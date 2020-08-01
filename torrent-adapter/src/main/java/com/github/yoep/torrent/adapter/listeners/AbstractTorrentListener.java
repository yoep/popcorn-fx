package com.github.yoep.torrent.adapter.listeners;

import com.github.yoep.torrent.adapter.model.DownloadStatus;
import com.github.yoep.torrent.adapter.state.TorrentState;

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
    public void onDownloadProgress(DownloadStatus status) {
        //no-op
    }

    @Override
    public void onPieceFinished(int pieceIndex) {
        //no-op
    }
}

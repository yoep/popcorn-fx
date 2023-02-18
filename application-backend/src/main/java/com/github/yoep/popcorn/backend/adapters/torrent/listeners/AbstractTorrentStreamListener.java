package com.github.yoep.popcorn.backend.adapters.torrent.listeners;

import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;

public abstract class AbstractTorrentStreamListener implements TorrentStreamListener {
    @Override
    public void onStateChanged(TorrentStreamState newState) {
        //no-op
    }

    @Override
    public void onStreamError(Exception error) {
        //no-op
    }

    @Override
    public void onStreamReady() {
        //no-op
    }

    @Override
    public void onStreamStopped() {
        //no-op
    }
}

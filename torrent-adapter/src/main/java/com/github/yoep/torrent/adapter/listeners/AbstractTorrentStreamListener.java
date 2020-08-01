package com.github.yoep.torrent.adapter.listeners;

import com.github.yoep.torrent.adapter.state.TorrentStreamState;

public abstract class AbstractTorrentStreamListener implements TorrentStreamListener {
    @Override
    public void onStateChanged(TorrentStreamState oldState, TorrentStreamState newState) {
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

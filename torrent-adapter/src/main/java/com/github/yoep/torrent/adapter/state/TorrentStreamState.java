package com.github.yoep.torrent.adapter.state;

public enum TorrentStreamState {
    /**
     * The torrent stream is being prepared.
     */
    PREPARING,
    /**
     * The torrent stream is streaming.
     */
    STREAMING,
    /**
     * The torrent stream has been stopped.
     */
    STOPPED
}

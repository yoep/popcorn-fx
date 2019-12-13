package com.github.yoep.popcorn.torrent.listeners;

import com.github.yoep.popcorn.torrent.StreamStatus;
import com.github.yoep.popcorn.torrent.Torrent;

public interface TorrentListener {
    /**
     * Invoked when the torrent failed to load.
     *
     * @param message The load error that occurred.
     */
    void onLoadError(String message);

    /**
     * Invoked when the torrent stream is started.
     *
     * @param torrent The torrent stream.
     */
    void onStreamStarted(Torrent torrent);

    void onStreamError(Torrent torrent, Exception e);

    void onStreamReady(Torrent torrent);

    void onStreamProgress(Torrent torrent, StreamStatus status);

    void onStreamStopped();
}

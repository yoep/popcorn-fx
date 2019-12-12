package com.github.yoep.popcorn.torrent.listeners;

import com.github.yoep.popcorn.torrent.StreamStatus;
import com.github.yoep.popcorn.torrent.Torrent;

public interface TorrentListener {
    void onStreamPrepared(Torrent torrent);

    void onStreamStarted(Torrent torrent);

    void onStreamError(Torrent torrent, Exception e);

    void onStreamReady(Torrent torrent);

    void onStreamProgress(Torrent torrent, StreamStatus status);

    void onStreamStopped();
}

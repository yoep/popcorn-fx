package com.github.yoep.popcorn.backend.adapters.torrent.listeners;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;

/**
 * A torrent listener which listens on events of a {@link TorrentStream}.
 */
public interface TorrentStreamListener {
    /**
     * Invoked when the torrent stream state is changed.
     *
     * @param newState The new stream state.
     */
    void onStateChanged(TorrentStreamState newState);

    /**
     * Invoked when an error occurred during the streaming of the torrent.
     *
     * @param error The error that occurred.
     */
    void onStreamError(Exception error);

    /**
     * Invoked when the torrent is ready to be streamed.
     */
    void onStreamReady();

    /**
     * Invoked when the torrent stream is being stopped.
     */
    void onStreamStopped();
}

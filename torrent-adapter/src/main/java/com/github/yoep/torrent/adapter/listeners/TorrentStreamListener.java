package com.github.yoep.torrent.adapter.listeners;

import com.github.yoep.torrent.adapter.state.TorrentStreamState;

/**
 * A torrent listener which listens on events of a {@link com.github.yoep.torrent.adapter.model.TorrentStream}.
 */
public interface TorrentStreamListener {
    /**
     * Invoked when the torrent stream state is changed.
     *
     * @param oldState The old stream state.
     * @param newState The new stream state.
     */
    void onStateChanged(TorrentStreamState oldState, TorrentStreamState newState);

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

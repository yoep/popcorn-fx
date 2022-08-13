package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;

import java.util.Optional;

/**
 * The {@link TorrentStreamService} manages the active served torrent streams which are available over HTTP.
 * Use this service to start/stop the stream of a {@link Torrent}.
 */
public interface TorrentStreamService {
    /**
     * Start the stream process for the given torrent.
     *
     * @param torrent The torrent to start streaming.
     * @return Returns the streaming torrent.
     */
    TorrentStream startStream(Torrent torrent);

    /**
     * Stop the stream for the given torrent.
     * This method removes the torrent from streaming.
     *
     * @param torrent The torrent stream to stop.
     */
    void stopStream(TorrentStream torrent);

    /**
     * Stop all the torrent streams which are currently running.
     */
    void stopAllStreams();

    /**
     * Resolve the given filename to an existing torrent stream.
     * This torrent stream can be streamed over HTTP.
     *
     * @param filename The filename to resolve to a torrent.
     * @return Returns the stream torrent if found, else null.
     */
    Optional<TorrentStream> resolve(String filename);
}

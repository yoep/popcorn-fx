package com.github.yoep.popcorn.backend.adapters.torrent.model;

import com.github.yoep.popcorn.backend.adapters.torrent.InvalidTorrentStreamException;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
import javafx.beans.property.ReadOnlyObjectProperty;
import org.springframework.core.io.Resource;

/**
 * Extension of {@link Torrent} is available for streaming.
 */
public interface TorrentStream extends Torrent {
    /**
     * Get the state of the stream.
     *
     * @return Returns the stream state.
     */
    TorrentStreamState getStreamState();

    /**
     * Get the stream state property.
     *
     * @return Returns the stream state property.
     */
    ReadOnlyObjectProperty<TorrentStreamState> streamStateProperty();

    /**
     * Get the underlying torrent of this torrent stream.
     *
     * @return Returns the torrent which is being streamed.
     */
    Torrent getTorrent();

    /**
     * Get the url on which the torrent stream can be accessed.
     *
     * @return Returns the stream url of the torrent.
     */
    String getStreamUrl();

    /**
     * Register a new listener for this torrent stream.
     *
     * @param listener The listener to register.
     */
    void addListener(TorrentStreamListener listener);

    /**
     * Remove a registered listener from this torrent stream.
     *
     * @param listener The listener to remove.
     */
    void removeListener(TorrentStreamListener listener);

    /**
     * Get the stream resource of this torrent stream.
     *
     * @return Returns the stream resource of the given torrent.
     */
    Resource stream() throws InvalidTorrentStreamException;

    /**
     * Stop this torrent stream.
     * This will set the stream state to {@link TorrentStreamState#STOPPED} and prevent any future stream of this torrent stream.
     */
    void stopStream();
}

package com.github.yoep.popcorn.torrent.listeners;

import com.github.yoep.popcorn.torrent.StreamStatus;
import com.github.yoep.popcorn.torrent.Torrent;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

public class TorrentListenerHolder implements TorrentListener {
    private final List<TorrentListener> listeners = new ArrayList<>();

    @Override
    public void onStreamStarted(Torrent torrent) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onStreamStarted(torrent));
        }
    }

    @Override
    public void onStreamError(Torrent torrent, Exception ex) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onStreamError(torrent, ex));
        }
    }

    @Override
    public void onStreamReady(Torrent torrent) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onStreamReady(torrent));
        }
    }

    @Override
    public void onStreamProgress(Torrent torrent, StreamStatus status) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onStreamProgress(torrent, status));
        }
    }

    @Override
    public void onStreamStopped() {
        synchronized (listeners) {
            listeners.forEach(TorrentListener::onStreamStopped);
        }
    }

    /**
     * Add the torrent listener to the listeners.
     *
     * @param listener The listener to add.
     */
    public void addListener(TorrentListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            this.listeners.add(listener);
        }
    }

    /**
     * Remove the torrent listener from the listeners.
     *
     * @param listener The listener to remove.
     */
    public void removeListener(TorrentListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.remove(listener);
        }
    }
}

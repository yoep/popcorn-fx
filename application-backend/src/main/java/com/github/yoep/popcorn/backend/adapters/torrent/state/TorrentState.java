package com.github.yoep.popcorn.backend.adapters.torrent.state;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

import java.util.Arrays;

/**
 * The torrent state.
 */
public enum TorrentState implements NativeMapped {
    /**
     * The torrent is currently being created.
     */
    INITIALIZING,
    /**
     * The torrent is currently verifying the existing files.
     */
    VERIFYING_FILES,
    /**
     * The torrent is currently retrieving the metadata from peers.
     */
    RETRIEVING_METADATA,
    /**
     * The torrent is ready to start the download process.
     */
    READY,
    /**
     * The torrent is starting the download process.
     */
    STARTING,
    /**
     * The torrent is currently downloading.
     */
    DOWNLOADING,
    /**
     * The torrent is currently paused.
     */
    PAUSED,
    /**
     * The torrent has completed the download.
     */
    COMPLETED,
    /**
     * The torrent encountered a fatal error and cannot continue.
     * This state is mostly encountered during the creation/start of the torrent.
     */
    ERROR;

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext context) {
        return Arrays.stream(values())
                .filter(e -> e.ordinal() == (int) nativeValue)
                .findFirst()
                .orElse(TorrentState.ERROR);
    }

    @Override
    public Object toNative() {
        return ordinal();
    }

    @Override
    public Class<?> nativeType() {
        return Integer.class;
    }
}

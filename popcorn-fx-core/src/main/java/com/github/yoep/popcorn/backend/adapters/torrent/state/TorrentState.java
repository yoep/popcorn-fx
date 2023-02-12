package com.github.yoep.popcorn.backend.adapters.torrent.state;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

/**
 * The torrent state.
 */
public enum TorrentState implements NativeMapped {
    /**
     * The torrent is currently being created.
     */
    CREATING,
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
        var ordinal = (Integer) nativeValue;
        if (ordinal == -1)
            return ERROR;

        return TorrentState.values()[ordinal];
    }

    @Override
    public Object toNative() {
        if (this == ERROR)
            return -1;

        return ordinal();
    }

    @Override
    public Class<?> nativeType() {
        return Integer.class;
    }
}

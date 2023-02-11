package com.github.yoep.popcorn.backend.adapters.torrent.state;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

public enum TorrentStreamState implements NativeMapped {
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
    STOPPED;

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext context) {
        var ordinal = (Integer) nativeValue;
        return TorrentStreamState.values()[ordinal];
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

package com.github.yoep.popcorn.backend.playlists;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

public enum PlaylistState implements NativeMapped {
    IDLE,
    STARTING,
    RETRIEVING_SUBTITLES,
    DOWNLOADING_SUBTITLE,
    CONNECTING,
    PLAYING,
    STOPPED,
    COMPLETED,
    ERROR;

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext context) {
        var oridinal = (int) nativeValue;
        return PlaylistState.values()[oridinal];
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

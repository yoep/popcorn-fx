package com.github.yoep.popcorn.backend.controls;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

public enum PlaybackControlEvent implements NativeMapped {
    TogglePlaybackState,
    Forward,
    Rewind;

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext context) {
        var ordinal = (int) nativeValue;
        return PlaybackControlEvent.values()[ordinal];
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

package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

public enum StartScreen implements NativeMapped {
    MOVIES,
    SERIES,
    FAVORITES;

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext context) {
        var ordinal = (int) nativeValue;
        return values()[ordinal];
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

package com.github.yoep.popcorn.backend.updater;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

public enum UpdateState implements NativeMapped {
    CHECKING_FOR_NEW_VERSION,
    UPDATE_AVAILABLE,
    NO_UPDATE_AVAILABLE,
    DOWNLOADING,
    DOWNLOAD_FINISHED,
    INSTALLING,
    ERROR;

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

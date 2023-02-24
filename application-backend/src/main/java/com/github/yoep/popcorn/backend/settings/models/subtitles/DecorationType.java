package com.github.yoep.popcorn.backend.settings.models.subtitles;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

public enum DecorationType implements NativeMapped {
    NONE,
    OUTLINE,
    OPAQUE_BACKGROUND,
    SEE_THROUGH_BACKGROUND;

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

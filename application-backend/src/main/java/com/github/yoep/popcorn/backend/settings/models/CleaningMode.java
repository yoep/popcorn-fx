package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

import java.util.Arrays;

public enum CleaningMode implements NativeMapped {
    OFF,
    ON_SHUTDOWN,
    WATCHED;

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext context) {
        return Arrays.stream(values())
                .filter(e -> e.ordinal() == (int) nativeValue)
                .findFirst()
                .orElse(null);
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

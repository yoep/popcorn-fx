package com.github.yoep.popcorn.backend.loader;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

import java.util.Arrays;

public enum LoaderState implements NativeMapped {
    INITIALIZING,
    STARTING,
    RETRIEVING_SUBTITLES,
    DOWNLOADING_SUBTITLE,
    RETRIEVING_METADATA,
    CONNECTING,
    DOWNLOADING,
    DOWNLOAD_FINISHED,
    READY,
    PLAYING;

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

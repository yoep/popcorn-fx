package com.github.yoep.popcorn.backend.media;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

import java.util.Arrays;

public enum MediaError implements NativeMapped {
    Failed,
    NoItemsFound,
    NoAvailableProviders;

    public String getMessage() {
        return switch (this) {
            case Failed -> "Failed to retrieve media information";
            case NoItemsFound -> "No media items could be found";
            case NoAvailableProviders -> "No providers are available for retrieving information";
        };
    }

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

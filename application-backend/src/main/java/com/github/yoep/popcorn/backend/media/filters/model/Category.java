package com.github.yoep.popcorn.backend.media.filters.model;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import lombok.Getter;

@Getter
public enum Category implements NativeMapped {
    MOVIES("movies"),
    SERIES("series"),
    FAVORITES("favorites");

    /**
     * The provider name for this category.
     */
    private final String providerName;

    Category(String providerName) {
        this.providerName = providerName;
    }

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

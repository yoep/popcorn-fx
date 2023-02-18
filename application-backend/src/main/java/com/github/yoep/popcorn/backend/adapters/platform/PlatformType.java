package com.github.yoep.popcorn.backend.adapters.platform;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import lombok.Getter;

@Getter
public enum PlatformType implements NativeMapped {
    WINDOWS("windows"),
    MAC("macos"),
    DEBIAN("debian");

    private final String name;

    PlatformType(String name) {
        this.name = name;
    }

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext fromNativeContext) {
        return PlatformType.values()[(Integer) nativeValue];
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
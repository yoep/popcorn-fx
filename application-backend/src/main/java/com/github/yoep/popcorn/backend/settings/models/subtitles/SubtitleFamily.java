package com.github.yoep.popcorn.backend.settings.models.subtitles;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import lombok.Getter;

@Getter
public enum SubtitleFamily implements NativeMapped {
    ARIAL("Arial"),
    COMIC_SANS("Comic Sans MS"),
    GEORGIA("Georgia"),
    TAHOMA("Tahoma"),
    TREBUCHET_MS("Trebuchet MS"),
    VERDANA("Verdana");

    private final String family;

    SubtitleFamily(String family) {
        this.family = family;
    }

    @Override
    public String toString() {
        return family;
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

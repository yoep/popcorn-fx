package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import lombok.Getter;

import java.util.Arrays;

/**
 * The subtitle format type from which it was parsed.
 */
@Getter
public enum SubtitleType implements NativeMapped {
    SRT("srt"),
    VTT("vtt");

    private final String extension;

    SubtitleType(String extension) {
        this.extension = extension;
    }

    /**
     * Get the parser type based on the given file extension.
     *
     * @param extension The file extension.
     * @return Returns the parser type for the given file extension.
     * @throws EnumConstantNotPresentException Is thrown when know suitable parser type could be found for the given extension.
     */
    public static SubtitleType fromExtension(String extension) {
        return Arrays.stream(SubtitleType.values())
                .filter(e -> e.getExtension().equals(extension))
                .findFirst()
                .orElseThrow(() -> new EnumConstantNotPresentException(SubtitleType.class, extension));
    }

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext context) {
        return SubtitleType.values()[(Integer) nativeValue];
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

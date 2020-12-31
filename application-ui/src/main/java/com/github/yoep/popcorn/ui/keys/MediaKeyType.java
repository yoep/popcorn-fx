package com.github.yoep.popcorn.ui.keys;

import lombok.Getter;

import java.util.Arrays;

@Getter
public enum MediaKeyType {
    UNKNOWN(-1),
    STOP(0),
    PLAY(1),
    PAUSE(2),
    PREVIOUS(3),
    NEXT(4),
    VOLUME_LOWER(5),
    VOLUME_HIGHER(6);

    private final int nativeValue;

    MediaKeyType(int nativeValue) {
        this.nativeValue = nativeValue;
    }

    /**
     * Get the media key type based on the given native value.
     *
     * @param value The native value to convert.
     * @return Returns the {@link MediaKeyType} for the native value.
     * @throws EnumConstantNotPresentException Is thrown when the native value couldn't be mapped.
     */
    public static MediaKeyType fromNativeValue(int value) {
        return Arrays.stream(MediaKeyType.values())
                .filter(e -> e.getNativeValue() == value)
                .findFirst()
                .orElseThrow(() -> new EnumConstantNotPresentException(MediaKeyType.class, String.valueOf(value)));
    }
}

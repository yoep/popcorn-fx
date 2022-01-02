package com.github.yoep.player.qt.player;

import lombok.Getter;

import java.util.Arrays;

@Getter
public enum PopcornPlayerState {
    UNKNOWN(-1),
    PLAYING(1),
    PAUSED(2),
    BUFFERING(3),
    STOPPED(0);

    private final int nativeValue;

    PopcornPlayerState(int nativeValue) {
        this.nativeValue = nativeValue;
    }

    /**
     * Get the {@link PopcornPlayerState} of the given native value.
     *
     * @param nativeValue The native state value.
     * @return Returns the popcorn player state for the given native value.
     * @throws EnumConstantNotPresentException Is thrown when the native value in unknown for this enum.
     */
    public static PopcornPlayerState of(int nativeValue) {
        return Arrays.stream(PopcornPlayerState.values())
                .filter(e -> e.getNativeValue() == nativeValue)
                .findFirst()
                .orElseThrow(() -> new EnumConstantNotPresentException(PopcornPlayerState.class, "nativeValue:" + nativeValue));
    }
}

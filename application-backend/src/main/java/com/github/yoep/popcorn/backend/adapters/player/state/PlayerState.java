package com.github.yoep.popcorn.backend.adapters.player.state;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;

/**
 * The state of the player.
 */
public enum PlayerState implements NativeMapped {
    READY,
    LOADING,
    BUFFERING,
    PLAYING,
    PAUSED,
    STOPPED,
    ERROR,
    UNKNOWN;

    public static final int UNKNOWN_ORDINAL = -1;

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext context) {
        var oridinal = (int) nativeValue;
        return (oridinal == UNKNOWN_ORDINAL) ? PlayerState.UNKNOWN : PlayerState.values()[oridinal];
    }

    @Override
    public Object toNative() {
        return this == UNKNOWN ? -1 : ordinal();
    }

    @Override
    public Class<?> nativeType() {
        return Integer.class;
    }
}

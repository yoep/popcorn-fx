package com.github.yoep.popcorn.backend.adapters.torrent.state;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import lombok.Getter;

@Getter
public enum TorrentHealthState implements NativeMapped {
    UNKNOWN("health_unknown", "unknown"),
    BAD("health_bad", "bad"),
    MEDIUM("health_medium", "medium"),
    GOOD("health_good", "good"),
    EXCELLENT("health_excellent", "excellent");

    private final String key;
    private final String styleClass;

    TorrentHealthState(String key, String styleClass) {
        this.key = key;
        this.styleClass = styleClass;
    }

    @Override
    public Object fromNative(Object nativeValue, FromNativeContext context) {
        var oridinal = (int) nativeValue;
        return TorrentHealthState.values()[oridinal];
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

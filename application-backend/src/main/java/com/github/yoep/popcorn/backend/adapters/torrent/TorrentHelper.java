package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Torrent;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class TorrentHelper {
    public static String getHealthStateKey(Torrent.Health.State state) {
        return switch (state) {
            case BAD -> "health_bad";
            case MEDIUM -> "health_medium";
            case GOOD -> "health_good";
            case EXCELLENT -> "health_excellent";
            default -> "health_unknown";
        };
    }

    public static String getHealthStateStyleClass(Torrent.Health.State state) {
        return switch (state) {
            case BAD -> "bad";
            case MEDIUM -> "medium";
            case GOOD -> "good";
            case EXCELLENT -> "excellent";
            default -> "unknown";
        };
    }
}

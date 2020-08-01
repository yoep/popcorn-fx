package com.github.yoep.torrent.adapter.state;

import lombok.Getter;

@Getter
public enum TorrentHealthState {
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
}

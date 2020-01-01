package com.github.yoep.popcorn.torrent.models;

import lombok.Getter;

@Getter
public class TorrentHealth {
    private final Status status;
    private final double ratio;
    private final int seeds;
    private final int peers;

    /**
     * Instantiate a new torrent health instance.
     *
     * @param status The health status of the torrent.
     * @param ratio  The health ratio.
     * @param seeds  The total torrent seeds.
     * @param peers  The total torrent peers.
     */
    public TorrentHealth(Status status, double ratio, int seeds, int peers) {
        this.status = status;
        this.ratio = ratio;
        this.seeds = seeds;
        this.peers = peers;
    }

    @Getter
    public enum Status {
        UNKNOWN("health_unknown", "unknown"),
        BAD("health_bad", "bad"),
        MEDIUM("health_medium", "medium"),
        GOOD("health_good", "good"),
        EXCELLENT("health_excellent", "excellent");

        private final String key;
        private final String styleClass;

        Status(String key, String styleClass) {
            this.key = key;
            this.styleClass = styleClass;
        }
    }
}

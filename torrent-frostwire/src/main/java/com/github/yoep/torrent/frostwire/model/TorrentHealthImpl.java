package com.github.yoep.torrent.frostwire.model;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentHealth;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentHealthState;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

@Getter
@ToString
@EqualsAndHashCode
public class TorrentHealthImpl implements TorrentHealth {
    private final TorrentHealthState state;
    private final double ratio;
    private final int seeds;
    private final int peers;

    /**
     * Instantiate a new torrent health instance.
     *
     * @param healthState The health state of the torrent.
     * @param ratio       The health ratio.
     * @param seeds       The total torrent seeds.
     * @param peers       The total torrent peers.
     */
    public TorrentHealthImpl(TorrentHealthState healthState, double ratio, int seeds, int peers) {
        this.state = healthState;
        this.ratio = ratio;
        this.seeds = seeds;
        this.peers = peers;
    }
}

package com.github.yoep.popcorn.torrent.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;

@Getter
@Builder
@AllArgsConstructor
public class StreamStatus {
    /**
     * A value in the range [0, 1], that represents the progress of the torrent's
     * current task. It may be checking files or downloading.
     */
    private final float progress;
    private final int bufferProgress;
    /**
     * The number of peers that are seeding that this client is currently connected to.
     */
    private final int seeds;
    /**
     * The total rates for all peers for this torrent. These will usually have better
     * precision than summing the rates from all peers. The rates are given as the
     * number of bytes per second.
     */
    private final int downloadSpeed;
    /**
     * The total rates for all peers for this torrent. These will usually have better
     * precision than summing the rates from all peers. The rates are given as the
     * number of bytes per second.
     */
    private final int uploadSpeed;
}

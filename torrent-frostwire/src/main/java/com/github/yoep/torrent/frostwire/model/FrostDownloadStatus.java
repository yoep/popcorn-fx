package com.github.yoep.torrent.frostwire.model;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import lombok.*;

@Getter
@Builder
@ToString
@EqualsAndHashCode
@AllArgsConstructor
public class FrostDownloadStatus implements DownloadStatus {
    /**
     * A value in the range [0, 1], that represents the progress of the torrent's
     * current task. It may be checking files or downloading.
     */
    private final float progress;
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
    /**
     * The number of bytes we have downloaded, only counting the pieces that we actually want
     * to download. i.e. excluding any pieces that we have but have priority 0 (i.e. not wanted).
     */
    private final long downloaded;
    /**
     * The total number of bytes we want to download. This may be smaller than the total
     * torrent size in case any pieces are prioritized to 0, i.e. not wanted.
     */
    private final long totalSize;
}

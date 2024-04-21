package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.util.Objects;

@Getter
@EqualsAndHashCode(callSuper = false)
public class ShowTorrentDetailsEvent extends ShowDetailsEvent {
    /**
     * The magnet uri that was used to resolve the torrent details.
     */
    private final String magnetUri;

    /**
     * The torrent info that needs to be shown.
     */
    private final TorrentInfo torrentInfo;

    public ShowTorrentDetailsEvent(Object source, String magnetUri, TorrentInfo torrentInfo) {
        super(source, null);
        Objects.requireNonNull(magnetUri, "magnetUri cannot be null");
        Objects.requireNonNull(torrentInfo, "torrentInfo cannot be null");
        this.magnetUri = magnetUri;
        this.torrentInfo = torrentInfo;
    }
}

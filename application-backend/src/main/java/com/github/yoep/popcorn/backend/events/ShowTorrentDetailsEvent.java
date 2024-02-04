package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.util.Assert;

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
        Assert.notNull(magnetUri, "magnetUri cannot be null");
        Assert.notNull(torrentInfo, "torrentInfo cannot be null");
        this.magnetUri = magnetUri;
        this.torrentInfo = torrentInfo;
    }
}

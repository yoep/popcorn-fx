package com.github.yoep.popcorn.ui.events;

import com.github.yoep.torrent.adapter.model.TorrentFileInfo;
import com.github.yoep.torrent.adapter.model.TorrentInfo;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class LoadUrlTorrentEvent extends LoadTorrentEvent {
    /**
     * The torrent to load.
     */
    private final TorrentInfo torrentInfo;
    /**
     * The torrent file info that has been selected.
     */
    private final TorrentFileInfo torrentFileInfo;

    public LoadUrlTorrentEvent(Object source, TorrentInfo torrentInfo, TorrentFileInfo torrentFileInfo) {
        super(source);
        Assert.notNull(torrentInfo, "torrentInfo cannot be null");
        Assert.notNull(torrentFileInfo, "torrentFileInfo cannot be null");
        this.torrentInfo = torrentInfo;
        this.torrentFileInfo = torrentFileInfo;
    }
}

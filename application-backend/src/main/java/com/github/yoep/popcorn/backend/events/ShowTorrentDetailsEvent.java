package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Torrent;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.util.Objects;

@Getter
@EqualsAndHashCode(callSuper = false)
public class ShowTorrentDetailsEvent extends ShowDetailsEvent<ShowDetails> {
    /**
     * The torrent info that needs to be shown.
     */
    private final Torrent.Info torrentInfo;

    public ShowTorrentDetailsEvent(Object source, Torrent.Info torrentInfo) {
        super(source, null);
        Objects.requireNonNull(torrentInfo, "torrentInfo cannot be null");
        this.torrentInfo = torrentInfo;
    }
}

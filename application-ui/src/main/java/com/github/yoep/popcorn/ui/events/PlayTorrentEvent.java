package com.github.yoep.popcorn.ui.events;

import com.github.yoep.torrent.adapter.model.Torrent;
import com.github.yoep.torrent.adapter.model.TorrentStream;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class PlayTorrentEvent extends PlayVideoEvent {
    /**
     * The torrent that needs to be played.
     */
    private final Torrent torrent;
    /**
     * The torrent stream that is being used for playback.
     */
    private final TorrentStream torrentStream;

    @Builder
    public PlayTorrentEvent(Object source, String url, String title, boolean subtitlesEnabled, Torrent torrent, TorrentStream torrentStream) {
        super(source, url, title, subtitlesEnabled);
        Assert.notNull(torrent, "torrent cannot be null");
        Assert.notNull(torrentStream, "torrentStream cannot be null");
        this.torrent = torrent;
        this.torrentStream = torrentStream;
    }
}

package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.util.Objects;

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

    @Builder(builderMethodName = "playTorrentBuilder")
    public PlayTorrentEvent(Object source, String url, String title, boolean subtitlesEnabled, String thumbnail, Torrent torrent, TorrentStream torrentStream) {
        super(source, url, title, subtitlesEnabled, thumbnail);
        Objects.requireNonNull(torrent, "torrent cannot be null");
        Objects.requireNonNull(torrentStream, "torrentStream cannot be null");
        this.torrent = torrent;
        this.torrentStream = torrentStream;
    }
}

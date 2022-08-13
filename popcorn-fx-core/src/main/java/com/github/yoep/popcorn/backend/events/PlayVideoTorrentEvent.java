package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import lombok.Builder;

public class PlayVideoTorrentEvent extends PlayTorrentEvent {
    @Builder(builderMethodName = "videoTorrentBuilder")
    public PlayVideoTorrentEvent(Object source, String url, String title, boolean subtitlesEnabled, Torrent torrent, TorrentStream torrentStream) {
        super(source, url, title, subtitlesEnabled, null, torrent, torrentStream);
    }
}

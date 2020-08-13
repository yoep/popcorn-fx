package com.github.yoep.popcorn.ui.events;

import com.github.yoep.torrent.adapter.model.Torrent;
import com.github.yoep.torrent.adapter.model.TorrentStream;
import lombok.Builder;

public class PlayVideoTorrentEvent extends PlayTorrentEvent {
    @Builder
    public PlayVideoTorrentEvent(Object source, String url, String title, boolean subtitlesEnabled, Torrent torrent, TorrentStream torrentStream) {
        super(source, url, title, subtitlesEnabled, torrent, torrentStream);
    }
}

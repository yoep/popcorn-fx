package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.lang.Nullable;
import org.springframework.util.Assert;

import java.util.Optional;

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
    /**
     * The subtitle that needs to be loaded while loading the torrent.
     */
    @Nullable
    private final SubtitleInfo subtitle;

    public LoadUrlTorrentEvent(Object source, TorrentInfo torrentInfo, TorrentFileInfo torrentFileInfo, @Nullable SubtitleInfo subtitle) {
        super(source);
        Assert.notNull(torrentInfo, "torrentInfo cannot be null");
        Assert.notNull(torrentFileInfo, "torrentFileInfo cannot be null");
        this.torrentInfo = torrentInfo;
        this.torrentFileInfo = torrentFileInfo;
        this.subtitle = subtitle;
    }

    public Optional<SubtitleInfo> getSubtitle() {
        return Optional.ofNullable(subtitle);
    }
}

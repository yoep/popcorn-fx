package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.lang.Nullable;
import org.springframework.util.Assert;

import java.util.Optional;

@Getter
@EqualsAndHashCode(callSuper = false)
public class LoadMediaTorrentEvent extends LoadTorrentEvent {
    /**
     * The selected torrent that needs to be preloaded.
     */
    private final MediaTorrentInfo torrent;
    /**
     * The media for which the torrent is being loaded.
     */
    private final Media media;
    /**
     * The sub item of the media to play;
     */
    @Nullable
    private final Media subItem;
    /**
     * The quality of the torrent that should is being loaded.
     */
    private final String quality;
    /**
     * The subtitle that needs to be loaded while loading the torrent.
     */
    @Nullable
    private final SubtitleInfo subtitle;

    @Builder
    public LoadMediaTorrentEvent(Object source, MediaTorrentInfo torrent, Media media, @Nullable Media subItem, String quality, @Nullable SubtitleInfo subtitle) {
        super(source);
        Assert.notNull(torrent, "torrent cannot be null");
        Assert.notNull(media, "media cannot be null");
        Assert.notNull(quality, "quality cannot be null");
        this.torrent = torrent;
        this.media = media;
        this.subItem = subItem;
        this.quality = quality;
        this.subtitle = subtitle;
    }

    public Optional<Media> getSubItem() {
        return Optional.ofNullable(subItem);
    }

    public Optional<SubtitleInfo> getSubtitle() {
        return Optional.ofNullable(subtitle);
    }
}

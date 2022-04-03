package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import org.springframework.lang.Nullable;

import java.util.Optional;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class PlayMediaEvent extends PlayTorrentEvent {
    /**
     * The media that needs to be played.
     */
    private final Media media;
    /**
     * The media sub item that needs to be played.
     */
    @Nullable
    private final Media subMediaItem;
    /**
     * The video quality of the media.
     */
    private final String quality;

    @Builder(builderMethodName = "mediaBuilder")
    public PlayMediaEvent(Object source,
                          String url,
                          String title,
                          boolean subtitlesEnabled,
                          Torrent torrent,
                          TorrentStream torrentStream,
                          Media media,
                          Media subMediaItem,
                          String quality) {
        super(source, url, title, subtitlesEnabled,  media.getImages().getFanart(), torrent, torrentStream);
        this.media = media;
        this.subMediaItem = subMediaItem;
        this.quality = quality;
    }

    public Optional<Media> getSubMediaItem() {
        return Optional.ofNullable(subMediaItem);
    }
}

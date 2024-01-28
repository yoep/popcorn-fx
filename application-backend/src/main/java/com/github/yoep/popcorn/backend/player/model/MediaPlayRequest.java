package com.github.yoep.popcorn.backend.player.model;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.Optional;

@Getter
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
public class MediaPlayRequest extends StreamPlayRequest {
    private final String quality;
    private final Media media;
    private final Media subMediaItem;

    @Builder(builderMethodName = "mediaBuilder")
    public MediaPlayRequest(String url, String title, String thumb, Long autoResumeTimestamp, TorrentStream torrentStream,
                            String quality, Media media, Media subMediaItem) {
        super(url, title, thumb, autoResumeTimestamp, torrentStream, true);
        this.quality = quality;
        this.media = media;
        this.subMediaItem = subMediaItem;
    }

    @Override
    public Optional<String> getQuality() {
        return Optional.ofNullable(quality);
    }

    /**
     * The sub media item to play.
     *
     * @return Returns the sub item if known, else {@link Optional#empty()}.
     */
    public Optional<Media> getSubMediaItem() {
        return Optional.ofNullable(subMediaItem);
    }
}

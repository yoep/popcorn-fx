package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.lang.Nullable;

import java.util.Optional;

@Getter
@EqualsAndHashCode(callSuper = false)
public class PlayMediaEvent extends PlayTorrentEvent {
    /**
     * The media that needs to be played.
     */
    private final Media media;
    /**
     * The video quality of the media.
     */
    private final String quality;
    /**
     * The subtitle that needs to be added to the playback of the media.
     * When no subtitle was selected, it will be by default {@link Subtitle#none()}.
     * If it is {@link Optional#empty()} it probably means that this activity is a trailer activity.
     */
    @Nullable
    private final Subtitle subtitle;

    @Builder(builderMethodName = "mediaBuilder")
    public PlayMediaEvent(Object source,
                          String url,
                          String title,
                          boolean subtitlesEnabled,
                          Torrent torrent,
                          TorrentStream torrentStream,
                          Media media,
                          String quality,
                          @Nullable Subtitle subtitle) {
        super(source, url, title, subtitlesEnabled, media.getImages().getFanart(), torrent, torrentStream);
        this.media = media;
        this.quality = quality;
        this.subtitle = subtitle;
    }

    public Optional<Subtitle> getSubtitle() {
        return Optional.ofNullable(subtitle);
    }
}

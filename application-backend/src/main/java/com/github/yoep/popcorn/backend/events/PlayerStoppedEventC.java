package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.MediaItem;
import com.sun.jna.Structure;
import com.sun.jna.ptr.LongByReference;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"url", "time", "duration", "media"})
public class PlayerStoppedEventC extends Structure implements Closeable {
    public static class ByValue extends PlayerStoppedEventC implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(String url, MediaItem media, Long time, Long duration) {
            super(url, media, time, duration);
        }
    }

    public String url;
    public LongByReference time;
    public LongByReference duration;
    public MediaItem.ByReference media;

    public PlayerStoppedEventC() {
    }

    private PlayerStoppedEventC(String url, MediaItem media, Long time, Long duration) {
        var mediaRef = new MediaItem.ByReference();

        if (media != null) {
            mediaRef.movieOverview = media.movieOverview;
            mediaRef.movieDetails = media.movieDetails;
            mediaRef.showDetails = media.showDetails;
            mediaRef.showOverview = media.showOverview;
            mediaRef.episode = media.episode;
        }

        this.url = url;
        this.media = mediaRef;
        this.time = Optional.ofNullable(time)
                .map(LongByReference::new)
                .orElse(null);
        this.duration = Optional.ofNullable(duration)
                .map(LongByReference::new)
                .orElse(null);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    public static PlayerStoppedEventC.ByValue from(PlayerStoppedEvent event) {
        var media = event.getMedia().map(MediaItem::from).orElse(null);
        var time = event.getTime() == PlayerStoppedEvent.UNKNOWN ? null : event.getTime();
        var duration = event.getDuration() == PlayerStoppedEvent.UNKNOWN ? null : event.getDuration();

        return new PlayerStoppedEventC.ByValue(
                event.getUrl(),
                media,
                time,
                duration
        );
    }
}

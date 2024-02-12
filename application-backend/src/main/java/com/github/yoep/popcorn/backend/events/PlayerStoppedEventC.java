package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.MediaItem;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
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
    }

    public String url;
    public Pointer time;
    public Pointer duration;
    public MediaItem.ByReference media;

    private Long cachedTime;
    private Long cachedDuration;

    public PlayerStoppedEventC() {
    }

    public Long getTime() {
        return cachedTime;
    }

    public Long getDuration() {
        return cachedDuration;
    }

    @Override
    public void read() {
        super.read();
        cachedTime = Optional.ofNullable(time)
                .map(e -> e.getLong(0))
                .orElse(null);
        cachedDuration = Optional.ofNullable(duration)
                .map(e -> e.getLong(0))
                .orElse(null);
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(media)
                .ifPresent(MediaItem::close);
    }
}

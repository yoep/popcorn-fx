package com.github.yoep.popcorn.backend.events;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"url", "title", "thumbnail", "quality", "autoResumeTimestamp", "subtitleEnabled"})
public class PlayerStartedEventC extends Structure implements Closeable {
    public static class ByValue extends PlayerStartedEventC implements Structure.ByValue {
    }

    public String url;
    public String title;
    public Pointer thumbnail;
    public Pointer quality;
    public Long autoResumeTimestamp;
    public byte subtitleEnabled;

    public Optional<String> getThumbnail() {
        return Optional.ofNullable(thumbnail)
                .map(e -> e.getString(0));
    }

    public Optional<String> getQuality() {
        return Optional.ofNullable(quality)
                .map(e -> e.getString(0));
    }

    public boolean isSubtitlesEnabled() {
        return subtitleEnabled == 1;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

package com.github.yoep.popcorn.backend.loader;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"url", "title", "thumbnail", "background", "quality"})
public class LoadingStartedEventC extends Structure implements Closeable {
    public static class ByValue extends LoadingStartedEventC implements Structure.ByValue {
    }

    public String url;
    public String title;
    public Pointer thumbnail;
    public Pointer background;
    public Pointer quality;

    private String cachedThumbnail;
    private String cachedBackground;

    public LoadingStartedEventC() {
    }

    public Optional<String> getThumbnail() {
        return Optional.ofNullable(cachedThumbnail);
    }

    public Optional<String> getBackground() {
        return Optional.ofNullable(cachedBackground);
    }

    public Optional<String> getQuality() {
        return Optional.ofNullable(quality)
                .map(e -> e.getString(0));
    }

    @Override
    public void read() {
        super.read();
        cachedThumbnail = Optional.ofNullable(thumbnail)
                .map(e -> e.getString(0))
                .orElse(null);
        cachedBackground = Optional.ofNullable(background)
                .map(e -> e.getString(0))
                .orElse(null);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

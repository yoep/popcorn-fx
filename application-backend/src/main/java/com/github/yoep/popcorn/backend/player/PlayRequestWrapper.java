package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.lib.Handle;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"url", "title", "caption", "thumb", "background", "quality", "autoResumeTimestamp", "streamHandle", "subtitle"})
public class PlayRequestWrapper extends Structure implements Closeable, PlayRequest {
    public static class ByValue extends PlayRequestWrapper implements Structure.ByValue {
    }

    public String url;
    public String title;
    public Pointer caption;
    public Pointer thumb;
    public Pointer background;
    public Pointer quality;
    public Pointer autoResumeTimestamp;
    public Pointer streamHandle;
    public PlaySubtitleRequest.ByValue subtitle;

    private String cachedCaption;
    private String cachedThumb;
    private String cachedBackground;
    private String cachedQuality;
    private Long cachedAutoResumeTimestamp;
    private Handle cachedStreamHandle;

    public boolean isSubtitlesEnabled() {
        return subtitle.enabled == 1;
    }

    @Override
    public Optional<String> getCaption() {
        return Optional.ofNullable(cachedCaption);
    }

    @Override
    public Optional<String> getThumbnail() {
        return Optional.ofNullable(cachedThumb);
    }

    @Override
    public Optional<String> getBackground() {
        return Optional.ofNullable(cachedBackground);
    }

    @Override
    public Optional<String> getQuality() {
        return Optional.ofNullable(cachedQuality);
    }

    @Override
    public Optional<Long> getAutoResumeTimestamp() {
        return Optional.ofNullable(cachedAutoResumeTimestamp);
    }

    @Override
    public Optional<Handle> getStreamHandle() {
        return Optional.ofNullable(cachedStreamHandle);
    }

    @Override
    public void read() {
        super.read();
        this.cachedCaption = Optional.ofNullable(caption)
                .map(e -> e.getString(0))
                .orElse(null);
        this.cachedThumb = Optional.ofNullable(thumb)
                .map(e -> e.getString(0))
                .orElse(null);
        this.cachedBackground = Optional.ofNullable(background)
                .map(e -> e.getString(0))
                .orElse(null);
        this.cachedQuality = Optional.ofNullable(quality)
                .map(e -> e.getString(0))
                .orElse(null);
        this.cachedAutoResumeTimestamp = Optional.ofNullable(autoResumeTimestamp)
                .map(e -> e.getLong(0))
                .orElse(null);
        this.cachedStreamHandle = Optional.ofNullable(streamHandle)
                .map(e -> e.getLong(0))
                .map(Handle::new)
                .orElse(null);
    }

    @Override
    public void close() {
        setAutoSynch(false);
        subtitle.close();
    }
}

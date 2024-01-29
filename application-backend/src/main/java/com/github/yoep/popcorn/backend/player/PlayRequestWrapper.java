package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"url", "title", "thumb", "autoResumeTimestamp", "subtitlesEnabled"})
public class PlayRequestWrapper extends Structure implements Closeable {
    public String url;
    public String title;
    public Pointer thumb;
    public Pointer autoResumeTimestamp;
    public byte subtitlesEnabled;

    private String cachedThumb;

    public PlayRequest toPlayRequest() {
        return SimplePlayRequest.builder()
                .url(url)
                .title(title)
                .thumb(cachedThumb)
                .autoResumeTimestamp(getAutoResumeTimestamp().orElse(0L))
                .subtitlesEnabled(isSubtitlesEnabled())
                .build();
    }

    public Optional<Long> getAutoResumeTimestamp() {
        return Optional.ofNullable(autoResumeTimestamp)
                .map(e -> e.getLong(0));
    }

    public boolean isSubtitlesEnabled() {
        return subtitlesEnabled == 1;
    }

    @Override
    public void read() {
        super.read();
        this.cachedThumb = Optional.ofNullable(thumb)
                .map(e -> e.getString(0))
                .orElse(null);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}

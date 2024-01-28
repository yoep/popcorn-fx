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
@Structure.FieldOrder({"url", "title", "thumb"})
public class PlayRequestWrapper extends Structure implements Closeable {
    public String url;
    public String title;
    public Pointer thumb;

    private String cachedThumb;

    public PlayRequest toPlayRequest() {
        return SimplePlayRequest.builder()
                .url(url)
                .title(title)
                .thumb(cachedThumb)
                .build();
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

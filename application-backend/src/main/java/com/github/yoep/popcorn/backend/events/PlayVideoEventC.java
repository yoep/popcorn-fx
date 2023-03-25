package com.github.yoep.popcorn.backend.events;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"url", "title", "thumb"})
public class PlayVideoEventC extends Structure implements Closeable {
    public static class ByValue extends PlayVideoEventC implements Structure.ByValue {
    }

    public String url;
    public String title;
    public String thumb;

    @Override
    public void close() {
        setAutoSynch(false);
    }

    public static PlayVideoEventC.ByValue from(PlayVideoEvent event) {
        var instance = new PlayVideoEventC.ByValue();
        instance.url = event.getUrl();
        instance.title = event.getTitle();
        instance.thumb = event.getThumbnail();
        return instance;
    }
}

package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"url", "title", "showName", "thumb"})
public class PlayVideoEventC extends Structure implements Closeable {
    public static class ByValue extends PlayVideoEventC implements Structure.ByValue {
    }

    public String url;
    public String title;
    public String showName;
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

    public static PlayVideoEventC.ByValue from(PlayMediaEvent event) {
        var instance = new PlayVideoEventC.ByValue();
        instance.url = event.getUrl();
        instance.title = event.getSubMediaItem()
                .map(Media::getTitle)
                .orElse(event.getTitle());
        instance.showName = event.getSubMediaItem()
                .map(e -> event.getMedia().getTitle())
                .orElse(null);
        instance.thumb = event.getSubMediaItem()
                .filter(e -> e instanceof Episode)
                .map(e -> (Episode) e)
                .flatMap(Episode::getThumb)
                .orElseGet(() -> event.getMedia()
                        .getImages()
                        .getPoster());
        return instance;
    }
}
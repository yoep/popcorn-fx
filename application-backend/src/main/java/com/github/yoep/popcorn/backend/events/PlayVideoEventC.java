package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"url", "title", "subtitle", "thumb"})
public class PlayVideoEventC extends Structure implements Closeable {
    public static class ByValue extends PlayVideoEventC implements Structure.ByValue {
    }

    /**
     * The video playback url
     */
    public String url;
    /**
     * The video title
     */
    public String title;
    /**
     * The video subtitle/additional info
     */
    public String subtitle;
    /**
     * The video thumbnail
     */
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
        instance.subtitle = event.getSubMediaItem()
                .map(e -> event.getMedia().getTitle())
                .orElseGet(() -> retrieveMovieSubtitle(event.getMedia()));
        instance.thumb = event.getSubMediaItem()
                .filter(e -> e instanceof Episode)
                .map(e -> (Episode) e)
                .flatMap(Episode::getThumb)
                .orElseGet(() -> event.getMedia()
                        .getImages()
                        .getPoster());
        return instance;
    }

    private static String retrieveMovieSubtitle(Media media) {
        if (media instanceof MovieOverview movie) {
            return movie.getYear();
        }

        return null;
    }
}

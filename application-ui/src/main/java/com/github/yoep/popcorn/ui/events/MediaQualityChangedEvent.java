package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import com.github.yoep.popcorn.backend.media.providers.Media;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.util.Objects;

/**
 * Invoked when the quality selection of a media item is changed.
 */
@Getter
@EqualsAndHashCode(callSuper = false)
public class MediaQualityChangedEvent extends ApplicationEvent {
    /**
     * The media item for which the quality changed.
     */
    private final Media media;
    /**
     * The new quality value for the media item.
     */
    private final String quality;

    public MediaQualityChangedEvent(Object source, Media media, String quality) {
        super(source);
        Objects.requireNonNull(media, "media cannot be null");
        Objects.requireNonNull(quality, "quality cannot be null");
        this.media = media;
        this.quality = quality;
    }
}

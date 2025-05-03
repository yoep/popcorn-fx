package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.Media;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

/**
 * Invoked when the details of a media item should be shown.
 */
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class ShowDetailsEvent<T extends Media> extends ApplicationEvent {
    private final T media;

    public ShowDetailsEvent(Object source, T media) {
        super(source);
        this.media = media;
    }
}

package com.github.yoep.popcorn.backend.events;

import org.springframework.context.ApplicationEvent;

/**
 * Invoked when the details of a media item should be shown.
 */
public class ShowDetailsEvent extends ApplicationEvent {
    public ShowDetailsEvent(Object source) {
        super(source);
    }
}

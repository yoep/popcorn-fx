package com.github.yoep.popcorn.backend.events;

import org.springframework.context.ApplicationEvent;

/**
 * Indicates that the watch now button has been clicked.
 */
public class WatchNowEvent extends ApplicationEvent {
    public WatchNowEvent(Object source) {
        super(source);
    }
}

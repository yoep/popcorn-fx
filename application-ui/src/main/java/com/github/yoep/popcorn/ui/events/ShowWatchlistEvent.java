package com.github.yoep.popcorn.ui.events;

import org.springframework.context.ApplicationEvent;

public class ShowWatchlistEvent extends ApplicationEvent {
    public ShowWatchlistEvent(Object source) {
        super(source);
    }
}

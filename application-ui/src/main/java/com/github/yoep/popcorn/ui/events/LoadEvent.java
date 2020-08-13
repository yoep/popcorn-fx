package com.github.yoep.popcorn.ui.events;

import org.springframework.context.ApplicationEvent;

public abstract class LoadEvent extends ApplicationEvent {
    public LoadEvent(Object source) {
        super(source);
    }
}

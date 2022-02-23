package com.github.yoep.popcorn.ui.events;

import org.springframework.context.ApplicationEvent;

public class ShowUpdateEvent extends ApplicationEvent {
    public ShowUpdateEvent(Object source) {
        super(source);
    }
}

package com.github.yoep.popcorn.ui.events;

import org.springframework.context.ApplicationEvent;

public class ShowAboutEvent extends ApplicationEvent {
    public ShowAboutEvent(Object source) {
        super(source);
    }
}

package com.github.yoep.popcorn.ui.events;

import org.springframework.context.ApplicationEvent;

public class ShowSettingsEvent extends ApplicationEvent {
    public ShowSettingsEvent(Object source) {
        super(source);
    }
}

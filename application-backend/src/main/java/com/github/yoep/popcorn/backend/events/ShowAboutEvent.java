package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;

@EqualsAndHashCode(callSuper = false)
public class ShowAboutEvent extends ApplicationEvent {
    public ShowAboutEvent(Object source) {
        super(source);
    }
}

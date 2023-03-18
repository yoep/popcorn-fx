package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

@EqualsAndHashCode(callSuper = false)
public class ShowAboutEvent extends ApplicationEvent {
    public ShowAboutEvent(Object source) {
        super(source);
    }
}

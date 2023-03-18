package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

@EqualsAndHashCode(callSuper = false)
public class CloseAboutEvent extends ApplicationEvent {
    public CloseAboutEvent(Object source) {
        super(source);
    }
}

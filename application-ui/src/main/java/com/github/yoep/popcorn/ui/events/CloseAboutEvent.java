package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

@EqualsAndHashCode(callSuper = false)
public class CloseAboutEvent extends ApplicationEvent {
    public CloseAboutEvent(Object source) {
        super(source);
    }
}

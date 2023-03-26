package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

@EqualsAndHashCode(callSuper = false)
public class CloseUpdateEvent extends ApplicationEvent {
    public CloseUpdateEvent(Object source) {
        super(source);
    }
}

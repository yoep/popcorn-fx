package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

@EqualsAndHashCode(callSuper = false)
public class CloseUpdateEvent extends ApplicationEvent {
    public CloseUpdateEvent(Object source) {
        super(source);
    }
}

package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

/**
 * Invoked when the update contents should be shown.
 */
@EqualsAndHashCode(callSuper = false)
public class ShowUpdateEvent extends ApplicationEvent {
    public ShowUpdateEvent(Object source) {
        super(source);
    }
}

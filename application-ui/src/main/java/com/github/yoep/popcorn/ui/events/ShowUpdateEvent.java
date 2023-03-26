package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

/**
 * Invoked when the update contents should be shown.
 */
@EqualsAndHashCode(callSuper = false)
public class ShowUpdateEvent extends ApplicationEvent {
    public ShowUpdateEvent(Object source) {
        super(source);
    }
}

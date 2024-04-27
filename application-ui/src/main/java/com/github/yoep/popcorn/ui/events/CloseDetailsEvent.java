package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

/**
 * Invoked when the details section is being closed by the user.
 */
@EqualsAndHashCode(callSuper = false)
public class CloseDetailsEvent extends ApplicationEvent {
    public CloseDetailsEvent(Object source) {
        super(source);
    }
}

package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

/**
 * Invoked when the details section is being closed by the user.
 */
@EqualsAndHashCode(callSuper = false)
public class CloseDetailsEvent extends ApplicationEvent {
    public CloseDetailsEvent(Object source) {
        super(source);
    }
}

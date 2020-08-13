package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

/**
 * Invoked when the load activity is being cancelled.
 */
@EqualsAndHashCode(callSuper = false)
public class CloseLoadEvent extends ApplicationEvent {
    public CloseLoadEvent(Object source) {
        super(source);
    }
}

package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

/**
 * Invoked when the load activity is being cancelled.
 */
@EqualsAndHashCode(callSuper = false)
public class CloseLoadEvent extends ApplicationEvent {
    public CloseLoadEvent(Object source) {
        super(source);
    }
}

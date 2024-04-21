package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

/**
 * Invoked when the content menu is being requested.
 */
@EqualsAndHashCode(callSuper = false)
public class ContextMenuEvent extends ApplicationEvent {
    public ContextMenuEvent(Object source) {
        super(source);
    }
}

package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

/**
 * Invoked when the content menu is being requested.
 */
@EqualsAndHashCode(callSuper = false)
public class ContextMenuEvent extends ApplicationEvent {
    public ContextMenuEvent(Object source) {
        super(source);
    }
}

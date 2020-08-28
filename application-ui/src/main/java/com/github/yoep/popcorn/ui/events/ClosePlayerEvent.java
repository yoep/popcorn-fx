package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.springframework.context.ApplicationEvent;

/**
 * Event indicating that the player is being closed.
 */
@ToString
@EqualsAndHashCode(callSuper = false)
public class ClosePlayerEvent extends ApplicationEvent {
    public ClosePlayerEvent(Object source) {
        super(source);
    }
}

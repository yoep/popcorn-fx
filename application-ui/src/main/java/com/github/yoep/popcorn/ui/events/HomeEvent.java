package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

/**
 * Invoked when request the home action.
 */
@EqualsAndHashCode
public class HomeEvent extends ApplicationEvent {
    public HomeEvent(Object source) {
        super(source);
    }
}

package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

/**
 * Invoked when request the home action.
 */
@EqualsAndHashCode
public class HomeEvent extends ApplicationEvent {
    public HomeEvent(Object source) {
        super(source);
    }
}

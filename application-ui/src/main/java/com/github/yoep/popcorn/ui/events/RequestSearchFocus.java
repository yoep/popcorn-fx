package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

/**
 * Invokes when a request is being made to focus the search input.
 */
@EqualsAndHashCode(callSuper = false)
public class RequestSearchFocus extends ApplicationEvent {
    public RequestSearchFocus(Object source) {
        super(source);
    }
}

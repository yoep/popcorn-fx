package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

/**
 * Invokes when a request is being made to focus the search input.
 */
@EqualsAndHashCode(callSuper = false)
public class RequestSearchFocus extends ApplicationEvent {
    public RequestSearchFocus(Object source) {
        super(source);
    }
}

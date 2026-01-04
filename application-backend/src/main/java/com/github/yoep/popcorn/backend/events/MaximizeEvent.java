package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

/**
 * Invoked when the window should be maximized or restored.
 */
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class MaximizeEvent extends ApplicationEvent {
    private final boolean maximize;

    public MaximizeEvent(Object source, boolean maximize) {
        super(source);
        this.maximize = maximize;
    }
}

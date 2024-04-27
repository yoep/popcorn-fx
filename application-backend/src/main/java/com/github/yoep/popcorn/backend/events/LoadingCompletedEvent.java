package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;
import lombok.ToString;

@ToString
@EqualsAndHashCode(callSuper = false)
public class LoadingCompletedEvent extends ApplicationEvent {
    public LoadingCompletedEvent(Object source) {
        super(source);
    }
}

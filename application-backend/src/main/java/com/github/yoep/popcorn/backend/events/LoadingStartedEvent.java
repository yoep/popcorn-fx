package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;
import lombok.ToString;

@ToString
@EqualsAndHashCode(callSuper = false)
public class LoadingStartedEvent extends ApplicationEvent {
    public LoadingStartedEvent(Object source) {
        super(source);
    }
}

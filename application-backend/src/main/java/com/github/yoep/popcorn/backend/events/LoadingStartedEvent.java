package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.springframework.context.ApplicationEvent;

@ToString
@EqualsAndHashCode(callSuper = false)
public class LoadingStartedEvent extends ApplicationEvent {
    public LoadingStartedEvent(Object source) {
        super(source);
    }
}

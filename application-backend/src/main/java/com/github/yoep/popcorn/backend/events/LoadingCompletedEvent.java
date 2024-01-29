package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.springframework.context.ApplicationEvent;

@ToString
@EqualsAndHashCode(callSuper = false)
public class LoadingCompletedEvent extends ApplicationEvent {
    public LoadingCompletedEvent(Object source) {
        super(source);
    }
}

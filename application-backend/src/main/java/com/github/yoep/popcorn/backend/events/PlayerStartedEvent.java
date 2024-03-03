package com.github.yoep.popcorn.backend.events;

import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import org.springframework.context.ApplicationEvent;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class PlayerStartedEvent extends ApplicationEvent {
    @Builder
    public PlayerStartedEvent(Object source) {
        super(source);
    }
}

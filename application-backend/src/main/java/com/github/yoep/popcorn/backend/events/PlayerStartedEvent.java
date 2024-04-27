package com.github.yoep.popcorn.backend.events;

import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class PlayerStartedEvent extends ApplicationEvent {
    @Builder
    public PlayerStartedEvent(Object source) {
        super(source);
    }
}

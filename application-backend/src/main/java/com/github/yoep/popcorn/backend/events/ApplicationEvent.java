package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.Objects;

@Getter
@ToString
@EqualsAndHashCode
public class ApplicationEvent {
    private final Object source;

    public ApplicationEvent(Object source) {
        Objects.requireNonNull(source, "source cannot be null");
        this.source = source;
    }
}

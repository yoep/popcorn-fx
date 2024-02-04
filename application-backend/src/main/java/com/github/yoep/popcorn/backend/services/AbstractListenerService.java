package com.github.yoep.popcorn.backend.services;

import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import javax.validation.constraints.NotNull;
import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

@Slf4j
@ToString
@EqualsAndHashCode
public abstract class AbstractListenerService<T> implements ListenerService<T> {
    protected final Queue<T> listeners = new ConcurrentLinkedQueue<>();

    @Override
    public void addListener(@NotNull T listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(T listener) {
        listeners.remove(listener);
    }

    protected void invokeListeners(Consumer<T> action) {
        listeners.forEach(e -> {
            try {
                action.accept(e);
            } catch (Exception ex) {
                log.warn("Failed to invoked listener {}, {}", e, ex.getMessage(), ex);
            }
        });
    }
}

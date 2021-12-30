package com.github.yoep.player.popcorn.services;

import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

public abstract class AbstractListenerService<T> {
    protected final Queue<T> listeners = new ConcurrentLinkedQueue<>();

    //region Methods

    public void addListener(T listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    public void removeListener(T listener) {
        listeners.remove(listener);
    }

    //endregion

    //region Functions

    protected void invokeListeners(Consumer<T> action) {
        listeners.forEach(action);
    }

    //endregion
}

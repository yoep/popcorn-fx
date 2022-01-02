package com.github.yoep.player.popcorn.services;

import javax.validation.constraints.NotNull;
import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

public abstract class AbstractListenerService<T> {
    protected final Queue<T> listeners = new ConcurrentLinkedQueue<>();

    //region Methods

    /**
     * Register the listener within the service.
     *
     * @param listener The listener to add.
     */
    public void addListener(@NotNull T listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    /**
     * Remove the listener from the service.
     *
     * @param listener The listener to remove.
     */
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

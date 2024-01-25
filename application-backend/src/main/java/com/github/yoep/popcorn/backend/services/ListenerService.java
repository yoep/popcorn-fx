package com.github.yoep.popcorn.backend.services;

public interface ListenerService<T> {
    /**
     * Register the listener within the service.
     *
     * @param listener The listener to add.
     */
    void addListener(T listener);

    /**
     * Remove the listener from the service.
     *
     * @param listener The listener to remove.
     */
    void removeListener(T listener);
}

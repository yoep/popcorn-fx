package com.github.yoep.popcorn.ui.keys;

public interface PopcornKeys {
    /**
     * Add the given listener to the media key press callbacks.
     *
     * @param listener The listener to register.
     */
    void addListener(MediaKeyPressedListener listener);

    /**
     * Release the popcorn keys resources.
     * This will destroy the pointer instance marking it invalid.
     */
    void release();
}

package com.github.yoep.popcorn.ui.keys;

public interface MediaKeyPressedListener {
    /**
     * Invoked when a media key has been pressed.
     *
     * @param type The media key type that was pressed.
     */
    void onMediaKeyPressed(MediaKeyType type);
}

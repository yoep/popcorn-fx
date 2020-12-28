package com.github.yoep.popcorn.ui.keys.bindings;

import com.sun.jna.Callback;

public interface popcorn_keys_media_key_pressed_t extends Callback {
    /**
     * The callback for when a media key has been pressed.
     *
     * @param mediaKeyType The media key type that was pressed.
     */
    void callback(int mediaKeyType);
}

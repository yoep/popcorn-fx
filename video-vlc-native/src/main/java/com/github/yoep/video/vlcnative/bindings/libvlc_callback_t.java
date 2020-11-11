package com.github.yoep.video.vlcnative.bindings;

import com.sun.jna.Callback;
import com.sun.jna.Pointer;

public interface libvlc_callback_t extends Callback {
    void callback(libvlc_event_t event, Pointer userData);
}

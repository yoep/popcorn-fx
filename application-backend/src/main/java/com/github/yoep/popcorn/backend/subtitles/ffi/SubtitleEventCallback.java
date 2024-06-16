package com.github.yoep.popcorn.backend.subtitles.ffi;

import com.sun.jna.Callback;

public interface SubtitleEventCallback extends Callback {
    void callback(SubtitleEvent.ByValue event);
}

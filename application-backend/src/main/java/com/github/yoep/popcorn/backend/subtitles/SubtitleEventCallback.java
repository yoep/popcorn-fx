package com.github.yoep.popcorn.backend.subtitles;

import com.sun.jna.Callback;

public interface SubtitleEventCallback extends Callback {
    void callback(SubtitleEvent.ByValue event);
}

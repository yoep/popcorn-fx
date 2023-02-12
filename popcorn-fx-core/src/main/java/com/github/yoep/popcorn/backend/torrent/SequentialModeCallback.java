package com.github.yoep.popcorn.backend.torrent;

import com.sun.jna.Callback;

interface SequentialModeCallback extends Callback {
    void callback();
}

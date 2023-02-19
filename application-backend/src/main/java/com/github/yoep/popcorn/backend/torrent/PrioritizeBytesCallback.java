package com.github.yoep.popcorn.backend.torrent;

import com.sun.jna.Callback;
import com.sun.jna.Pointer;

interface PrioritizeBytesCallback extends Callback {
    void callback(int len, Pointer ptr);
}

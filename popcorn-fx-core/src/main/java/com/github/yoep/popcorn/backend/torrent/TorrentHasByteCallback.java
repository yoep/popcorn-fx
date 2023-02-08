package com.github.yoep.popcorn.backend.torrent;

import com.sun.jna.Callback;
import com.sun.jna.Pointer;

public interface TorrentHasByteCallback extends Callback {
    byte callback(int len, Pointer bytes);
}

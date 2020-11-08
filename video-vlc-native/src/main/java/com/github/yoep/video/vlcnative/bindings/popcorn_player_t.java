package com.github.yoep.video.vlcnative.bindings;

import com.sun.jna.Pointer;
import com.sun.jna.PointerType;

public class popcorn_player_t extends PointerType {
    public popcorn_player_t() {
    }

    public popcorn_player_t(Pointer p) {
        super(p);
    }
}

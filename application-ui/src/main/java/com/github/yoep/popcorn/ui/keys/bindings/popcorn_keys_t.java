package com.github.yoep.popcorn.ui.keys.bindings;

import com.sun.jna.Pointer;
import com.sun.jna.PointerType;

public class popcorn_keys_t extends PointerType {
    public popcorn_keys_t() {
    }

    public popcorn_keys_t(Pointer p) {
        super(p);
    }
}

package com.github.yoep.popcorn.platform;

import com.sun.jna.Pointer;
import com.sun.jna.PointerType;

public class PlatformC extends PointerType {
    public PlatformC() {
    }

    public PlatformC(Pointer p) {
        super(p);
    }
}

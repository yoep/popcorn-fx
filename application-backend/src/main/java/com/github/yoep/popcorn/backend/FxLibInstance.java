package com.github.yoep.popcorn.backend;

import com.sun.jna.Native;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class FxLibInstance {
    public static final FxLibInstance INSTANCE = new FxLibInstance();

    private final FxLib NATIVE_INSTANCE = Native.load("popcorn_fx", FxLib.class);

    public FxLib get() {
        return NATIVE_INSTANCE;
    }
}

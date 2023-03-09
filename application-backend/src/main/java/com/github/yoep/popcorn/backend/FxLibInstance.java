package com.github.yoep.popcorn.backend;

import com.sun.jna.Library;
import com.sun.jna.Native;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.nio.charset.StandardCharsets;
import java.util.Collections;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class FxLibInstance {
    public static final FxLibInstance INSTANCE = new FxLibInstance();

    private final FxLib NATIVE_INSTANCE = Native.load("popcorn_fx", FxLib.class,
            Collections.singletonMap(Library.OPTION_STRING_ENCODING, StandardCharsets.UTF_8.name()));

    public FxLib get() {
        return NATIVE_INSTANCE;
    }
}

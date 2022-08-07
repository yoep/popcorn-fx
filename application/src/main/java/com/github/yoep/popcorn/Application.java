package com.github.yoep.popcorn;

import com.sun.jna.Library;
import com.sun.jna.Native;

public interface Application extends Library {
    Application INSTANCE = Native.load("application", Application.class);

    PopcornFx new_popcorn_fx();

    void delete_popcorn_fx(PopcornFx instance);
}

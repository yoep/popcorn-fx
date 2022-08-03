package com.github.yoep.popcorn;

import com.sun.jna.Library;
import com.sun.jna.Native;

public interface Application extends Library {
    Application INSTANCE = Native.load("application", Application.class);

    PopcornFx new_instance();
}

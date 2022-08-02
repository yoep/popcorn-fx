package com.github.yoep.popcorn.backend;

import com.sun.jna.Library;
import com.sun.jna.Native;

public interface ApplicationBackend extends Library {
    ApplicationBackend INSTANCE = Native.load("application_backend", ApplicationBackend.class);

    PopcornFx new_instance();
}

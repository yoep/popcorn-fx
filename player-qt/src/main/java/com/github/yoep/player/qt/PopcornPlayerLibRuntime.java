package com.github.yoep.player.qt;

import com.sun.jna.Platform;

public class PopcornPlayerLibRuntime {
    static final String LIBRARY_NAME_WINDOWS = "libPopcornPlayer";
    static final String LIBRARY_NAME_UNIX = "PopcornPlayer";

    private PopcornPlayerLibRuntime() {
    }

    public static String getLibraryName() {
        return Platform.isWindows() ? LIBRARY_NAME_WINDOWS : LIBRARY_NAME_UNIX;
    }
}

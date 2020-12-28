package com.github.yoep.video.vlcnative;

import com.sun.jna.Platform;

public class PopcornPlayerLibRuntime {
    private static final String LIBRARY_NAME_WINDOWS = "libPopcornPlayer";
    private static final String LIBRARY_NAME_UNIX = "PopcornPlayer";

    private PopcornPlayerLibRuntime() {
    }

    public static String getLibraryName() {
        return Platform.isWindows() ? LIBRARY_NAME_WINDOWS : LIBRARY_NAME_UNIX;
    }
}

package com.github.yoep.video.vlcnative;

public class PopcornPlayerLibRuntime {
    private static final String OS_NAME = System.getProperty("os.name").toLowerCase();
    private static final String WINDOWS_OS_INDICATOR = "win";
    private static final String LIBRARY_NAME_WINDOWS = "libPopcornPlayer";
    private static final String LIBRARY_NAME_UNIX = "PopcornPlayer";

    private PopcornPlayerLibRuntime() {
    }

    public static String getLibraryName() {
        return isWindows() ? LIBRARY_NAME_WINDOWS : LIBRARY_NAME_UNIX;
    }

    private static boolean isWindows() {
        return OS_NAME.contains(WINDOWS_OS_INDICATOR);
    }
}

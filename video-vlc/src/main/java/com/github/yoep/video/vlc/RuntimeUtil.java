package com.github.yoep.video.vlc;

public final class RuntimeUtil {
    private static final String OS_NAME = System.getProperty("os.name").toLowerCase();

    /**
     * Prevent direct instantiation by others.
     */
    private RuntimeUtil() {
    }

    /**
     * Test whether the runtime operating system is "unix-like".
     *
     * @return true if the runtime OS is unix-like, Linux, Unix, FreeBSD etc
     */
    public static boolean isNix() {
        return OS_NAME.contains("nux") || OS_NAME.contains("nix") || OS_NAME.contains("freebsd");
    }

    /**
     * Test whether the runtime operating system is a Windows variant.
     *
     * @return true if the runtime OS is Windows
     */
    public static boolean isWindows() {
        return OS_NAME.contains("win");
    }

    /**
     * Test whether the runtime operating system is a Mac variant.
     *
     * @return true if the runtime OS is Mac
     */
    public static boolean isMac() {
        return OS_NAME.contains("mac");
    }

    public static String getLibVlcCoreLibraryName() {
        return isWindows() ? "libvlccore" : "vlccore";
    }
}

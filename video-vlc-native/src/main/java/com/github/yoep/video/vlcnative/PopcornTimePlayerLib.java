package com.github.yoep.video.vlcnative;

import com.github.yoep.video.vlcnative.bindings.popcorn_desktop_player_t;
import com.sun.jna.Native;
import com.sun.jna.NativeLibrary;

public class PopcornTimePlayerLib {
    private static final String OS_NAME = System.getProperty("os.name").toLowerCase();
    private static final String WINDOWS_OS_INDICATOR = "win";
    private static final String LIBRARY_NAME_WINDOWS = "libPopcornDesktopPlayer";
    private static final String LIBRARY_NAME_UNIX = "PopcornDesktopPlayer";

    //region Constructors

    static {
        var libraryName = getLibraryName();

        NativeLibrary.addSearchPath(libraryName, "C:\\projects\\popcorn-desktop-javafx\\cmake-build-debug-mingw-64\\video-vlc-native\\src\\native");
        Native.register(libraryName);
    }

    private PopcornTimePlayerLib() {
    }

    //endregion

    //region Methods

    public static native popcorn_desktop_player_t popcorn_desktop_player_new();

    public static native int popcorn_desktop_player_exec(popcorn_desktop_player_t instance);

    public static native void popcorn_desktop_player_release(popcorn_desktop_player_t instance);

    public static native void popcorn_desktop_player_play(popcorn_desktop_player_t instance, String mrl);

    public static native void popcorn_desktop_player_pause(popcorn_desktop_player_t instance);

    public static native void popcorn_desktop_player_resume(popcorn_desktop_player_t instance);

    public static native void popcorn_desktop_player_stop(popcorn_desktop_player_t instance);

    public static native void popcorn_desktop_player_show(popcorn_desktop_player_t instance);

    public static native void popcorn_desktop_player_show_maximized(popcorn_desktop_player_t instance);

    //endregion

    //region Functions

    private static String getLibraryName() {
        return isWindows() ? LIBRARY_NAME_WINDOWS : LIBRARY_NAME_UNIX;
    }

    private static boolean isWindows() {
        return OS_NAME.contains(WINDOWS_OS_INDICATOR);
    }

    //endregion
}

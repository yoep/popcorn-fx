package com.github.yoep.video.vlcnative;

import com.github.yoep.video.vlcnative.bindings.popcorn_desktop_player;
import com.sun.jna.Native;
import com.sun.jna.StringArray;

public class PopcornTimePlayerLib {
    static {
        Native.register("libPopcornDesktopPlayer");
    }

    private PopcornTimePlayerLib() {
    }

    public static native popcorn_desktop_player popcorn_desktop_new(int argc, StringArray argv);
}

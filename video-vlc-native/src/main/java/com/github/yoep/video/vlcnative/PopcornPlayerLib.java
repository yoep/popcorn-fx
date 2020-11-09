package com.github.yoep.video.vlcnative;

import com.github.yoep.video.vlcnative.bindings.popcorn_player_t;
import com.sun.jna.Native;

public class PopcornPlayerLib {

    //region Constructors

    static {
        Native.register(PopcornPlayerLibRuntime.getLibraryName());
    }

    private PopcornPlayerLib() {
    }

    //endregion

    //region Methods

    public static native popcorn_player_t popcorn_player_new();

    public static native int popcorn_player_exec(popcorn_player_t instance);

    public static native void popcorn_player_release(popcorn_player_t instance);

    public static native void popcorn_player_play(popcorn_player_t instance, String mrl);

    public static native void popcorn_player_pause(popcorn_player_t instance);

    public static native void popcorn_player_resume(popcorn_player_t instance);

    public static native void popcorn_player_stop(popcorn_player_t instance);

    public static native void popcorn_player_show(popcorn_player_t instance);

    public static native void popcorn_player_show_maximized(popcorn_player_t instance);

    //endregion
}

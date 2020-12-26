package com.github.yoep.video.vlcnative;

import com.github.yoep.video.vlcnative.bindings.popcorn_player_duration_callback_t;
import com.github.yoep.video.vlcnative.bindings.popcorn_player_state_callback_t;
import com.github.yoep.video.vlcnative.bindings.popcorn_player_t;
import com.github.yoep.video.vlcnative.bindings.popcorn_player_time_callback_t;
import com.sun.jna.Native;
import com.sun.jna.StringArray;

public class PopcornPlayerLib {

    //region Constructors

    static {
        Native.register(PopcornPlayerLibRuntime.getLibraryName());
    }

    private PopcornPlayerLib() {
    }

    //endregion

    //region Methods

    public static native popcorn_player_t popcorn_player_new(int argc, StringArray argv);

    public static native void popcorn_player_release(popcorn_player_t instance);

    public static native void popcorn_player_play(popcorn_player_t instance, String mrl);

    public static native void popcorn_player_seek(popcorn_player_t instance, String time);

    public static native void popcorn_player_pause(popcorn_player_t instance);

    public static native void popcorn_player_resume(popcorn_player_t instance);

    public static native void popcorn_player_stop(popcorn_player_t instance);

    public static native void popcorn_player_show(popcorn_player_t instance);

    public static native void popcorn_player_fullscreen(popcorn_player_t instance, boolean fullscreen);

    public static native void popcorn_player_subtitle(popcorn_player_t instance, String url);

    public static native void popcorn_player_subtitle_delay(popcorn_player_t instance, long delay);

    public static native void popcorn_player_state_callback(popcorn_player_t instance, popcorn_player_state_callback_t callback);

    public static native void popcorn_player_time_callback(popcorn_player_t instance, popcorn_player_time_callback_t callback);

    public static native void popcorn_player_duration_callback(popcorn_player_t instance, popcorn_player_duration_callback_t callback);

    //endregion
}

package com.github.yoep.popcorn.ui.keys;

import com.github.yoep.popcorn.ui.keys.bindings.popcorn_keys_media_key_pressed_t;
import com.github.yoep.popcorn.ui.keys.bindings.popcorn_keys_t;
import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.StringArray;

public class PopcornKeysLib implements Library {

    //region Constructors

    static {
        Native.register(PopcornKeysLibDiscovery.LIBRARY_NAME);
    }

    private PopcornKeysLib() {
    }

    //endregion

    //region Methods

    /**
     * Initialize a new instance of Popcorn Keys.
     *
     * @param argc The total arguments for the library.
     * @param argv The arguments for the library.
     * @return Returns the pointer to the popcorn keys instance.
     */
    public static native popcorn_keys_t popcorn_keys_new(int argc, StringArray argv);

    /**
     * Release the Popcorn Keys instance.
     *
     * @param pk The Popcorn Keys instance.
     */
    public static native void popcorn_keys_release(popcorn_keys_t pk);

    public static native void popcorn_keys_media_callback(popcorn_keys_t pk, popcorn_keys_media_key_pressed_t callback);

    //endregion
}

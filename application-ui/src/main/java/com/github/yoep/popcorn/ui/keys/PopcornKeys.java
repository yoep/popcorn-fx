package com.github.yoep.popcorn.ui.keys;

import com.github.yoep.popcorn.ui.keys.bindings.popcorn_keys_t;
import com.sun.jna.StringArray;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public class PopcornKeys {
    private final popcorn_keys_t instance;

    /**
     * Initialize a new {@link PopcornKeys} instance.
     *
     * @param args The library arguments.
     */
    public PopcornKeys(String... args) {
        log.trace("Initializing new popcorn keys instance");
        instance = PopcornKeysLib.popcorn_keys_new(args.length, new StringArray(args));

        if (instance == null) {
            throw new PopcornKeysException("Failed to initialize Popcorn Keys instance");
        }

        log.debug("Popcorn keys has been initialized");
    }

    /**
     * Release the popcorn keys resources.
     * This will destroy the pointer instance marking it invalid.
     */
    public void release() {
        log.debug("Releasing Popcorn Keys");
        PopcornKeysLib.popcorn_keys_release(instance);
    }
}

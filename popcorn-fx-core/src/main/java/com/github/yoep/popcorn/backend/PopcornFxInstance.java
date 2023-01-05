package com.github.yoep.popcorn.backend;

import lombok.extern.slf4j.Slf4j;

@Slf4j
public class PopcornFxInstance {
    public static final PopcornFxInstance INSTANCE = new PopcornFxInstance();

    private final PopcornFx NATIVE_INSTANCE = FxLib.INSTANCE.new_popcorn_fx();

    private boolean valid;

    private PopcornFxInstance() {
        valid = true;
        log.info("Created new native Popcorn FX instance");
    }

    /**
     * Retrieve the FX native instance.
     * @return Return the FX native instance.
     */
    public PopcornFx get() {
        if (!valid) {
            throw new RuntimeException("The Popcorn FX instance is no longer valid");
        }

        return NATIVE_INSTANCE;
    }
}

package com.github.yoep.popcorn;

import lombok.extern.slf4j.Slf4j;

@Slf4j
public class PopcornFxManager {
    public static final PopcornFxManager INSTANCE = new PopcornFxManager();

    private final PopcornFx NATIVE_INSTANCE = Application.INSTANCE.new_popcorn_fx();

    private boolean valid;

    private PopcornFxManager() {
        valid = true;
        log.info("Created new native Popcorn FX instance");
    }

    /**
     * Retrieve the FX native instance.
     * @return Return the FX native instance.
     */
    public PopcornFx fxInstance() {
        if (!valid) {
            throw new RuntimeException("The Popcorn FX instance is no longer valid");
        }

        return NATIVE_INSTANCE;
    }

    /**
     * Dispose the FX native instance.
     */
    public void dispose() {
        log.info("Disposing native Popcorn FX instance");
        valid = false;
        Application.INSTANCE.dispose_popcorn_fx(NATIVE_INSTANCE);
    }
}

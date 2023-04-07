package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.PopcornFx;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class PopcornFxInstance {
    public static final PopcornFxInstance INSTANCE = new PopcornFxInstance();
    private final AtomicReference<PopcornFx> NATIVE_INSTANCE = new AtomicReference<>();

    /**
     * Retrieve the FX native instance.
     *
     * @return Return the FX native instance.
     */
    public PopcornFx get() {
        return NATIVE_INSTANCE.get();
    }

    public void set(PopcornFx instance) {
        if (NATIVE_INSTANCE.get() != null) {
            log.warn("Popcorn FX instance has already been set");
            return;
        }

        NATIVE_INSTANCE.set(instance);
    }
}

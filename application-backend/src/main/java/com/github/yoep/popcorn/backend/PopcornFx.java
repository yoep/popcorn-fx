package com.github.yoep.popcorn.backend;

import com.sun.jna.PointerType;
import lombok.extern.slf4j.Slf4j;

/**
 * The Popcorn FX instance.
 */
@Slf4j
public class PopcornFx extends PointerType {
    public void dispose() {
        log.debug("Disposing native Popcorn FX instance");
    }
}

package com.github.yoep.popcorn.backend.environment;

/**
 * Provider which supplies/check certain aspects of the environment.
 * This is mainly a wrapper class around the {@link javafx.application.Platform}.
 */
public interface PlatformProvider {

    /**
     * Verify if transparent windows are supported on the current runtime.
     *
     * @return Returns true if supported, else false.
     */
    boolean isTransparentWindowSupported();
}

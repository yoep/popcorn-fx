package com.github.yoep.popcorn.backend.adapters.platform;

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

    /**
     * Run the given action on the rendering thread of the platform.
     *
     * @param runnable The action to execute on the rendering thread.
     */
    void runOnRenderer(Runnable runnable);

    /**
     * Disable the screensaver function on the platform.
     */
    void disableScreensaver();
}

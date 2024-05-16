package com.github.yoep.popcorn.backend.adapters.platform;

/**
 * Provider which supplies/checks certain aspects of the current platform/environment.
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
     * Verify if the current platform is a MacOS system.
     * @return Returns true when the current platform is Mac, else false.
     */
    boolean isMac();

    /**
     * Exit the application in a safe manner.
     *
     * @param code The exit code of the application.
     */
    void exit(int code);
}

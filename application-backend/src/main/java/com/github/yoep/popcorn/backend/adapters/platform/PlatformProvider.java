package com.github.yoep.popcorn.backend.adapters.platform;

import javax.validation.constraints.NotNull;
import java.nio.file.Path;

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
     * Retrieve the current platform information.
     *
     * @return Returns the detected platform information.
     */
    PlatformInfo platformInfo();

    /**
     * Run the given action on the rendering thread of the platform.
     *
     * @param runnable The action to execute on the rendering thread.
     */
    void runOnRenderer(Runnable runnable);

    /**
     * Launch the given process path on the current platform.
     *
     * @param path The process path that needs to be started.
     * @return Returns true if the process path was launched with success, else false.
     */
    boolean launch(@NotNull Path path);

    /**
     * Launch the given command on the current platform.
     *
     * @param command The process command that needs to be started.
     * @return Returns true if the command was launched with success, else false.
     */
    boolean launch(@NotNull String command);

    /**
     * Exit the application in a safe manner.
     */
    void exit();
}

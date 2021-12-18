package com.github.yoep.popcorn.backend.adapters.screen;

import javafx.beans.property.ReadOnlyBooleanProperty;

/**
 * The screen service manages the state of the screen.
 * It has the ability to modify the state of the application within the screen.
 */
public interface ScreenService {
    /**
     * Check if the application is currently in fullscreen mode.
     *
     * @return Returns true if fullscreen is active, else false.
     */
    boolean isFullscreen();

    /**
     * Get the fullscreen property of the application.
     *
     * @return Returns the fullscreen property.
     */
    ReadOnlyBooleanProperty fullscreenProperty();

    /**
     * Toggle the fullscreen state of the application.
     */
    void toggleFullscreen();

    /**
     * The fullscreen state of the application.
     *
     * @param isFullscreenEnabled Set if the application should be in fullscreen or not.
     */
    void fullscreen(boolean isFullscreenEnabled);
}

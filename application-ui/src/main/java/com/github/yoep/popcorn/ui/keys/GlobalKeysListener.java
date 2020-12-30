package com.github.yoep.popcorn.ui.keys;

/**
 * The global keys listener which listens to keystrokes of global keys which cannot be intercepted by JavaFX.
 */
public interface GlobalKeysListener {
    /**
     * Invoked when the audio play key is pressed.
     */
    void onMediaPlay();

    /**
     * Invoked when the audio pause key is pressed.
     */
    void onMediaPause();

    /**
     * Invoked when the audio stop key is pressed.
     */
    void onMediaStop();

    /**
     * Invoked when the audio previous key is pressed.
     */
    void onPreviousMedia();

    /**
     * Invoked when the audio next key is pressed.
     */
    void onNextMedia();
}

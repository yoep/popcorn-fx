package com.github.yoep.player.qt.player;

import java.io.File;

public interface PopcornPlayer {
    /**
     * Show the popcorn player.
     */
    void show();

    /**
     * Set the fullscreen mode of the popcorn player.
     *
     * @param fullscreen The indication if the player should be shown in fullscreen.
     */
    void fullscreen(boolean fullscreen);

    /**
     * Play the given mrl in this player.
     *
     * @param mrl The mrl to play.
     */
    void play(String mrl);

    /**
     * Seek the given time within the current media playback.
     * This has no effect if no media is currently being played.
     *
     * @param time The time to seek in millis.
     */
    void seek(long time);

    /**
     * Pause the current media playback.
     * This has no effect if no media is currently playing.
     */
    void pause();

    /**
     * Resume the current media playback.
     * This has no effect if no media is currently playing.
     */
    void resume();

    /**
     * Stop the current media playback.
     * If no media is currently playing, it will have no effect on the media player.
     * In case the player window is visible, it will be still hidden even if there was currently no media being played.
     */
    void stop();

    /**
     * The subtitle file for the current media playback.
     *
     * @param file The .srt file to add to the current playback.
     */
    void subtitleFile(File file);

    /**
     * The subtitle file delay for the current media playback.
     *
     * @param delay The delay in milliseconds.
     */
    void subtitleDelay(long delay);

    /**
     * Register the given listener to this player instance.
     *
     * @param listener The listener to register (non-null).
     */
    void addListener(PopcornPlayerEventListener listener);

    /**
     * Remove an existing listener from this player.
     *
     * @param listener The player to remove from this listener.
     */
    void removeListener(PopcornPlayerEventListener listener);

    /**
     * Release the popcorn player instance.
     */
    void release();
}

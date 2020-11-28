package com.github.yoep.video.vlcnative.player;

import com.github.yoep.video.vlcnative.PopcornPlayerLib;
import com.github.yoep.video.vlcnative.bindings.popcorn_player_t;
import com.sun.jna.StringArray;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public class PopcornPlayer {
    private final popcorn_player_t instance;

    /**
     * Initialize a new {@link PopcornPlayer} instance.
     *
     * @param args The arguments for the {@link PopcornPlayer}.
     * @throws PopcornPlayerException Is thrown when the {@link PopcornPlayer} library could not be initialized.
     */
    public PopcornPlayer(String... args) {
        log.trace("Initializing new popcorn player library instance");
        instance = PopcornPlayerLib.popcorn_player_new(args.length, new StringArray(args));

        if (instance == null) {
            throw new PopcornPlayerException("Failed to initialize Popcorn Player");
        }
    }

    /**
     * Show the popcorn player.
     */
    public void show() {
        PopcornPlayerLib.popcorn_player_show(instance);
    }

    /**
     * Set the fullscreen mode of the popcorn player.
     *
     * @param fullscreen The indication if the player should be shown in fullscreen.
     */
    public void fullscreen(boolean fullscreen) {
        PopcornPlayerLib.popcorn_player_fullscreen(instance, fullscreen);
    }

    /**
     * Play the given mrl in this player.
     *
     * @param mrl The mrl to play.
     */
    public void play(String mrl) {
        PopcornPlayerLib.popcorn_player_play(instance, mrl);
    }

    /**
     * Pause the current media playback.
     * This has no effect if no media is currently playing.
     */
    public void pause() {
        PopcornPlayerLib.popcorn_player_pause(instance);
    }

    /**
     * Resume the current media playback.
     * This has no effect if no media is currently playing.
     */
    public void resume() {
        PopcornPlayerLib.popcorn_player_resume(instance);
    }

    /**
     * Stop the current media playback.
     * If no media is currently playing, it will have no effect on the media player.
     * In case the player window is visible, it will be still hidden even if there was currently no media being played.
     */
    public void stop() {
        PopcornPlayerLib.popcorn_player_stop(instance);
    }

    /**
     * Release the popcorn player instance.
     */
    public void release() {
        PopcornPlayerLib.popcorn_player_release(instance);
    }
}

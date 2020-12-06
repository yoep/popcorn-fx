package com.github.yoep.video.vlcnative.player;

import com.github.yoep.video.vlcnative.PopcornPlayerLib;
import com.github.yoep.video.vlcnative.bindings.popcorn_player_t;
import com.sun.jna.StringArray;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.io.File;
import java.util.Objects;

@Slf4j
public class PopcornPlayer {
    private final popcorn_player_t instance;
    private final PopcornPlayerEventManager eventManager;

    private String subtitleFileUri;

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

        this.eventManager = new PopcornPlayerEventManager(instance);
        log.debug("Popcorn player library initialized");
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
     * Seek the given time within the current media playback.
     * This has no effect if no media is currently being played.
     *
     * @param time The time to seek in millis.
     */
    public void seek(long time) {
        // normalize the time to 0 if the time is smaller than 0
        PopcornPlayerLib.popcorn_player_seek(instance, Math.max(time, 0));
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
     * The subtitle file for the current media playback.
     *
     * @param file The .srt file to add to the current playback.
     */
    public void subtitleFile(File file) {
        var subtitleFileUri = file.toURI().toASCIIString();

        if (Objects.equals(this.subtitleFileUri, subtitleFileUri)) {
            log.trace("Subtitle file \"{}\" has already been added to the media playback, ignoring action", subtitleFileUri);
            return;
        }

        log.debug("Adding subtitle file \"{}\" to the current media playback", file.getAbsolutePath());
        this.subtitleFileUri = subtitleFileUri;
        PopcornPlayerLib.popcorn_player_subtitle(instance, toLocalFileUri(this.subtitleFileUri));
    }

    /**
     * Register the given listener to this player instance.
     *
     * @param listener The listener to register (non-null).
     */
    public void addListener(PopcornPlayerEventListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        eventManager.addListener(listener);
    }

    /**
     * Remove an existing listener from this player.
     *
     * @param listener The player to remove from this listener.
     */
    public void removeListener(PopcornPlayerEventListener listener) {
        eventManager.removeListener(listener);
    }

    /**
     * Release the popcorn player instance.
     */
    public void release() {
        log.debug("Releasing popcorn player");
        PopcornPlayerLib.popcorn_player_release(instance);
    }

    private String toLocalFileUri(String uri) {
        if (uri.startsWith("file://")) {
            return uri;
        } else {
            return uri.replaceFirst("file:/", "file:///");
        }
    }
}

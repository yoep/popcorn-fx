package com.github.yoep.player.qt.player;

import com.github.yoep.player.qt.PopcornPlayerLib;
import com.github.yoep.player.qt.bindings.popcorn_player_t;
import com.sun.jna.StringArray;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.io.File;
import java.util.Objects;

@Slf4j
public class PopcornQtPlayer implements PopcornPlayer {
    private final popcorn_player_t instance;
    private final PopcornPlayerEventManager eventManager;

    private String subtitleFileUri;

    /**
     * Initialize a new {@link PopcornQtPlayer} instance.
     *
     * @param args The arguments for the {@link PopcornQtPlayer}.
     * @throws PopcornPlayerException Is thrown when the {@link PopcornQtPlayer} library could not be initialized.
     */
    public PopcornQtPlayer(String... args) {
        log.trace("Initializing new popcorn player library instance");
        instance = PopcornPlayerLib.popcorn_player_new(args.length, new StringArray(args));

        if (instance == null) {
            throw new PopcornPlayerException("Failed to initialize Popcorn Player");
        }

        this.eventManager = new PopcornPlayerEventManager(instance);
        log.debug("Popcorn player library initialized");
    }

    @Override
    public void show() {
        PopcornPlayerLib.popcorn_player_show(instance);
    }

    @Override
    public void fullscreen(boolean fullscreen) {
        PopcornPlayerLib.popcorn_player_fullscreen(instance, fullscreen);
    }

    @Override
    public void play(String mrl) {
        PopcornPlayerLib.popcorn_player_play(instance, mrl);
    }

    @Override
    public void seek(long time) {
        // normalize the time to 0 if the time is smaller than 0
        var seekTime = Math.max(time, 0);
        PopcornPlayerLib.popcorn_player_seek(instance, String.valueOf(seekTime));
    }

    @Override
    public void pause() {
        PopcornPlayerLib.popcorn_player_pause(instance);
    }

    @Override
    public void resume() {
        PopcornPlayerLib.popcorn_player_resume(instance);
    }

    @Override
    public void stop() {
        PopcornPlayerLib.popcorn_player_stop(instance);
    }

    @Override
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

    @Override
    public void subtitleDelay(long delay) {
        var microSeconds = delay * 1000;

        PopcornPlayerLib.popcorn_player_subtitle_delay(instance, microSeconds);
    }

    @Override
    public void addListener(PopcornPlayerEventListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        eventManager.addListener(listener);
    }

    @Override
    public void removeListener(PopcornPlayerEventListener listener) {
        eventManager.removeListener(listener);
    }

    @Override
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

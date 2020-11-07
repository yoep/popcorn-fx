package com.github.yoep.video.vlcnative;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import com.sun.jna.StringArray;
import javafx.beans.property.*;
import javafx.scene.Node;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;

import javax.annotation.PostConstruct;
import java.io.File;

public class VideoPlayerVlcNative implements VideoPlayer {
    private static final Pane videoSurfaceTracker = new StackPane();

    private final ObjectProperty<PlayerState> playerState = new SimpleObjectProperty<>(this, PLAYER_STATE_PROPERTY, PlayerState.UNKNOWN);
    private final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY);
    private final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY);

    private boolean initialized;

    //region VideoPlayer

    @Override
    public PlayerState getPlayerState() {
        return playerState.get();
    }

    @Override
    public ReadOnlyObjectProperty<PlayerState> playerStateProperty() {
        return playerState;
    }

    @Override
    public long getTime() {
        return time.get();
    }

    @Override
    public ReadOnlyLongProperty timeProperty() {
        return time;
    }

    @Override
    public long getDuration() {
        return duration.get();
    }

    @Override
    public ReadOnlyLongProperty durationProperty() {
        return duration;
    }

    @Override
    public boolean supports(String url) {
        return true;
    }

    @Override
    public boolean isInitialized() {
        return initialized;
    }

    @Override
    public Throwable getError() {
        return null;
    }

    @Override
    public Node getVideoSurface() {
        return videoSurfaceTracker;
    }

    @Override
    public void dispose() {

    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {

    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {

    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {

    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {

    }

    @Override
    public void stop() {

    }

    @Override
    public boolean supportsNativeSubtitleFile() {
        return true;
    }

    @Override
    public void subtitleFile(File file) {

    }

    @Override
    public void subtitleDelay(long delay) {

    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        new Thread(() -> {
            PopcornTimePlayerLib.popcorn_desktop_new(0, new StringArray(new String[0]));

            initialized = true;
        }).start();
    }

    //endregion
}

package com.github.yoep.video.vlcnative;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import com.github.yoep.video.vlcnative.bindings.popcorn_player_t;
import javafx.beans.property.*;
import javafx.scene.Group;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;
import javafx.stage.Stage;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.io.File;

@Slf4j
public class VideoPlayerVlcNative implements VideoPlayer {
    private static final Pane videoSurfaceTracker = new StackPane();

    private final ObjectProperty<PlayerState> playerState = new SimpleObjectProperty<>(this, PLAYER_STATE_PROPERTY, PlayerState.UNKNOWN);
    private final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY);
    private final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY);

    private popcorn_player_t instance;
    private boolean initialized;
    private boolean boundToWindow;

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
        if (instance != null) {
            log.debug("Releasing the native VLC player");
            PopcornPlayerLib.popcorn_player_release(instance);
        }
    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {
        checkInitialized();
        PopcornPlayerLib.popcorn_player_show_maximized(instance);
        PopcornPlayerLib.popcorn_player_play(instance, url);
    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {
        checkInitialized();
        PopcornPlayerLib.popcorn_player_pause(instance);
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        checkInitialized();
        PopcornPlayerLib.popcorn_player_resume(instance);
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {

    }

    @Override
    public void stop() {
        checkInitialized();
        PopcornPlayerLib.popcorn_player_stop(instance);
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
        log.trace("Initializing VLC native player");
        try {
            initializeTracker();
            new Thread(() -> {
                try {
                    instance = PopcornPlayerLib.popcorn_player_new();

                    if (instance == null) {
                        throw new VideoPlayerException("Failed to initialize native VLC player");
                    }

                    initialized = true;

                    var result = PopcornPlayerLib.popcorn_player_exec(instance);
                    log.debug("Qt Application exited with {}", result);
                } catch (Exception ex) {
                    log.error(ex.getMessage(), ex);
                }
            }, "QtThread").start();
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    //endregion

    //region Functions

    private void checkInitialized() {
        if (!initialized) {
            throw new VideoPlayerException("VLC native player has not yet been initialized");
        }
    }

    private void initializeTracker() {
        videoSurfaceTracker.sceneProperty().addListener((observableScene, oldValueScene, newValueScene) -> {
            if (newValueScene != null) {
                var stage = (Stage) newValueScene.getWindow();

                if (!boundToWindow)
                    bindFrameToWindow(newValueScene);

                stage.setAlwaysOnTop(true);
            } else if (oldValueScene != null) {
                var stage = (Stage) oldValueScene.getWindow();

                stage.setAlwaysOnTop(false);
            }
        });

    }

    private void bindFrameToWindow(Scene scene) {
        updateTransparentComponents(scene);

        boundToWindow = true;
        log.debug("ARM video player has been bound to the JavaFX window");
    }

    private void updateTransparentComponents(Scene scene) {
        var videoView = videoSurfaceTracker.getParent();
        var playerPane = videoView.getParent();
        var root = (Group) scene.getRoot();
        var mainPane = root.getChildren().get(0);

        playerPane.setStyle("-fx-background-color: transparent");
        mainPane.setStyle("-fx-background-color: transparent");
    }

    //endregion
}

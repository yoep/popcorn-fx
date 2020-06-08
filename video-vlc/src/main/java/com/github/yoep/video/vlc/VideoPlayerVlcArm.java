package com.github.yoep.video.vlc;

import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.scene.Group;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;
import javafx.stage.Stage;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.embedded.fullscreen.libvlc.LibVlcNativeFullScreenStrategy;

import javax.annotation.PostConstruct;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = true)
public class VideoPlayerVlcArm extends AbstractVideoPlayer<MediaPlayer> {
    private static final Pane videoSurfaceTracker = new StackPane();

    private boolean boundToWindow;

    //region Constructors

    public VideoPlayerVlcArm(NativeDiscovery nativeDiscovery) {
        super(nativeDiscovery);
        mediaPlayer = mediaPlayerFactory.mediaPlayers().newMediaPlayer();
    }

    //endregion

    //region Getters

    @Override
    public Node getVideoSurface() {
        return videoSurfaceTracker;
    }

    //endregion

    //region VideoPlayer

    @Override
    public void dispose() {
        if (mediaPlayer != null)
            mediaPlayer.release();
        if (mediaPlayerFactory != null)
            mediaPlayerFactory.release();

        this.mediaPlayer = null;
    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {
        checkInitialized();

        try {
            log.debug("Playing \"{}\" on VLC ARM video player", url);
            invokeOnVlc(() -> {
                mediaPlayer.media().start(url);
//                new LibVlcNativeFullScreenStrategy(mediaPlayer.mediaPlayerInstance()).enterFullScreenMode();
            });
        } catch (Exception ex) {
            log.error("Failed to play media on VLC ARM, " + ex.getMessage(), ex);
            setError(ex);
        }
    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {
        checkInitialized();
        invokeOnVlc(() -> mediaPlayer.controls().pause());
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        checkInitialized();
        invokeOnVlc(() -> mediaPlayer.controls().play());
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {
        checkInitialized();
        invokeOnVlc(() -> mediaPlayer.controls().setTime(time));
    }

    @Override
    public void stop() {
        checkInitialized();

        // check if the player state is already stopped
        // if so, ignore this action
        if (getPlayerState() == PlayerState.STOPPED)
            return;

        invokeOnVlc(() -> mediaPlayer.controls().stop());
        reset();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing VLC ARM player");

        try {
            mediaPlayer = mediaPlayerFactory.mediaPlayers().newMediaPlayer();

            initialize();
            initializeListeners();
            initializeTracker();
            initialized = true;
            log.trace("VLC player ARM initialization done");
        } catch (Exception ex) {
            log.error("Failed to initialize VLC ARM player, " + ex.getMessage(), ex);
            setError(new VideoPlayerException(ex.getMessage(), ex));
        }
    }

    //endregion

    //region Functions

    @Override
    protected MediaPlayerFactory createFactory(NativeDiscovery nativeDiscovery) {
        return new MediaPlayerFactory(nativeDiscovery,
                "--intf=dummy", "--no-video-deco", "--no-embedded-video", "--no-video-title-show", "--video-wallpaper");
    }

    private void initializeListeners() {
        playerStateProperty().addListener((observable, oldValue, newValue) -> {

        });
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

package com.github.yoep.video.vlc;

import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.scene.Group;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;
import javafx.scene.transform.Scale;
import javafx.scene.transform.Transform;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import uk.co.caprica.vlcj.player.component.EmbeddedMediaPlayerComponent;

import javax.annotation.PostConstruct;
import javax.swing.*;
import java.awt.*;
import java.awt.event.*;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = true)
public class VideoPlayerVlcArm extends AbstractVideoPlayer {
    private static final Pane videoSurfaceTracker = new StackPane();

    private JFrame frame;
    private EmbeddedMediaPlayerComponent mediaPlayerComponent;

    private boolean boundToWindow;

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
        if (frame != null)
            frame.dispose();

        this.frame = null;
        this.mediaPlayer = null;
        this.mediaPlayerComponent = null;
    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {
        checkInitialized();

        try {
            frame.setVisible(true);
            frame.setExtendedState(Frame.MAXIMIZED_BOTH);
            frame.toBack();

            mediaPlayer.media().play(url, VLC_OPTIONS);
        } catch (Exception ex) {
            log.error("Failed to play media on VLC ARM, " + ex.getMessage(), ex);
            setError(ex);
        }
    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {
        checkInitialized();
        mediaPlayer.controls().pause();
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        checkInitialized();
        mediaPlayer.controls().play();
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {
        checkInitialized();
        mediaPlayer.controls().setTime(time);
    }

    @Override
    public void stop() {
        checkInitialized();
        mediaPlayer.controls().stop();
        frame.setVisible(false);
        reset();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing VLC ARM player");

        try {
            mediaPlayerComponent = new EmbeddedMediaPlayerComponent();
            mediaPlayer = mediaPlayerComponent.mediaPlayer();

            initialize();
            initializeFrame();
            initializeFrameTracker();
            initialized = true;
            log.trace("VLC player ARM initialization done");
        } catch (Exception ex) {
            log.error("Failed to initialize VLC ARM player, " + ex.getMessage(), ex);
            setError(new VideoPlayerException(ex.getMessage(), ex));
        }
    }

    //endregion

    //region Functions

    private void initializeFrame() {
        frame = new JFrame("");

        frame.setDefaultCloseOperation(WindowConstants.HIDE_ON_CLOSE);
        frame.setUndecorated(true);
        frame.setContentPane(mediaPlayerComponent);
        frame.setAutoRequestFocus(false);
        frame.setFocusableWindowState(false);

        initializeFrameListeners();
    }

    private void initializeFrameListeners() {
        frame.addWindowListener(new WindowAdapter() {
            @Override
            public void windowClosing(WindowEvent e) {
                log.debug("ARM video player window is closing, stopping media playback");
                stop();
            }
        });

        frame.addKeyListener(new KeyAdapter() {
            @Override
            public void keyPressed(KeyEvent e) {
                switch (e.getKeyCode()) {
                    case KeyEvent.VK_ESCAPE:
                        stop();
                        break;
                    case KeyEvent.VK_P:
                    case KeyEvent.VK_SPACE:
                        togglePlayState();
                        break;
                }
            }
        });
        frame.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseClicked(MouseEvent e) {
                togglePlayState();
            }
        });
    }

    private void initializeFrameTracker() {
        videoSurfaceTracker.widthProperty().addListener((observable, oldValue, newValue) -> resizeFrame());
        videoSurfaceTracker.heightProperty().addListener((observable, oldValue, newValue) -> resizeFrame());

        videoSurfaceTracker.sceneProperty().addListener((observableScene, oldValueScene, newValueScene) -> {
            var scene = videoSurfaceTracker.getScene();

            if (scene != null && !boundToWindow) {
                var window = scene.getWindow();

                updateTransparentComponents(scene);
                window.xProperty().addListener((observable, oldValue, newValue) -> repositionFrame(scene));
                window.yProperty().addListener((observable, oldValue, newValue) -> repositionFrame(scene));

                resizeFrame();
                repositionFrame(scene);

                boundToWindow = true;
                log.debug("ARM video player has been bound to the JavaFX window");
            }
        });

    }

    private void resizeFrame() {
        log.trace("Resizing the ARM video player frame");
        var scene = videoSurfaceTracker.getScene();
        var trackerWidth = videoSurfaceTracker.getWidth();
        var trackerHeight = videoSurfaceTracker.getHeight();
        var scaleX = 1.0;
        var scaleY = 1.0;

        for (Transform transform : scene.getRoot().getChildrenUnmodifiable().get(0).getTransforms()) {
            if (transform instanceof Scale) {
                var scale = (Scale) transform;

                scaleX = scale.getX();
                scaleY = scale.getY();
                log.trace("Found scene scaling, using scale {}x{}", scaleX, scaleY);
                break;
            }
        }

        var width = (int) (trackerWidth * scaleX);
        var height = (int) (trackerHeight * scaleY);
        log.trace("Updating ARM video player size to {}x{}", width, height);
        frame.setSize(new Dimension(width, height));
    }

    private void repositionFrame(Scene scene) {
        log.trace("Repositioning ARM video player frame");
        var window = scene.getWindow();
        var x = (int) (window.getX() + scene.getX());
        var y = (int) (window.getY() + scene.getY());

        log.trace("Updating ARM video player position to {},{}", x, y);
        frame.setLocation(x, y);
    }

    private void updateTransparentComponents(Scene scene) {
        var videoView = videoSurfaceTracker.getParent();
        var playerPane = videoView.getParent();
        var root = (Group) scene.getRoot();
        var mainPane = root.getChildren().get(0);

        playerPane.setStyle("-fx-background-color: transparent");
        mainPane.setStyle("-fx-background-color: transparent");
    }

    private void togglePlayState() {
        if (getPlayerState() == PlayerState.PAUSED) {
            resume();
        } else if (getPlayerState() == PlayerState.PLAYING) {
            pause();
        }

        // ignore all other states
    }

    //endregion
}

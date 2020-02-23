package com.github.yoep.video.vlc;

import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import javafx.scene.Node;
import javafx.scene.layout.Pane;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import uk.co.caprica.vlcj.player.component.EmbeddedMediaPlayerComponent;

import javax.annotation.PostConstruct;
import javax.swing.*;
import java.awt.*;
import java.awt.event.WindowAdapter;
import java.awt.event.WindowEvent;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = true)
public class VideoPlayerVlcArm extends AbstractVideoPlayer {
    private JFrame frame;
    private EmbeddedMediaPlayerComponent mediaPlayerComponent;

    //region Getters

    @Override
    public Node getVideoSurface() {
        return new Pane();
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
        frame.setExtendedState(JFrame.MAXIMIZED_BOTH);
        frame.setUndecorated(true);
        frame.setType(Window.Type.UTILITY);
        frame.setMinimumSize(new Dimension(800, 600));
        frame.setContentPane(mediaPlayerComponent);

        frame.addWindowListener(new WindowAdapter() {
            @Override
            public void windowClosing(WindowEvent e) {
                log.debug("ARM video player window is closing, stopping media playback");
                stop();
            }
        });
    }

    //endregion
}

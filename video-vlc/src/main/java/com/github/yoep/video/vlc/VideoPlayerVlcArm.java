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
        if (frame != null)
            frame.dispose();
        if (mediaPlayer != null)
            mediaPlayer.release();
        if (mediaPlayerComponent != null)
            mediaPlayerComponent.release();

        this.frame = null;
        this.mediaPlayer = null;
        this.mediaPlayerComponent = null;
    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {
        checkInitialized();

        try {
            mediaPlayerComponent = new EmbeddedMediaPlayerComponent();

            frame = new JFrame();
            frame.setExtendedState(JFrame.MAXIMIZED_BOTH);
            frame.setUndecorated(true);
            frame.setContentPane(mediaPlayerComponent);

            mediaPlayer = mediaPlayerComponent.mediaPlayer();
            initialize();

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

        dispose();
        reset();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing VLC ARM player");

        try {
            initialized = true;
            log.trace("VLC player ARM initialization done");
        } catch (Exception ex) {
            log.error("Failed to initialize VLC ARM player, " + ex.getMessage(), ex);
            setError(new VideoPlayerException(ex.getMessage(), ex));
        }
    }

    //endregion
}

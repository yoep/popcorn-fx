package com.github.yoep.video.vlc;

import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.vlc.callback.FXBufferFormatCallback;
import com.github.yoep.video.vlc.callback.FXCallbackVideoSurface;
import com.github.yoep.video.vlc.callback.FXRenderCallback;
import javafx.scene.Node;
import javafx.scene.canvas.Canvas;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;

import javax.annotation.PostConstruct;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = true)
public class VideoPlayerVlc extends AbstractVideoPlayer {
    private final Canvas canvas = new Canvas();

    private final FXCallbackVideoSurface surface;
    private final MediaPlayerFactory mediaPlayerFactory;
    private final VideoAnimationTimer timer;

    //region Constructors

    /**
     * Instantiate a new video player.
     */
    public VideoPlayerVlc() {
        surface = new FXCallbackVideoSurface(new FXRenderCallback(canvas, new FXBufferFormatCallback()));
        mediaPlayerFactory = new MediaPlayerFactory();
        mediaPlayer = mediaPlayerFactory.mediaPlayers().newEmbeddedMediaPlayer();
        timer = new VideoAnimationTimer(surface::render);

        initialize();
    }

    //endregion

    //region Getters

    @Override
    public Node getVideoSurface() {
        return canvas;
    }

    //endregion

    //region VideoPlayer

    @Override
    public void dispose() {
        stop();
        mediaPlayer.release();
        mediaPlayerFactory.release();
    }

    @Override
    public void play(String url) {
        checkInitialized();

        timer.start();
        invokeOnVlc(() -> mediaPlayer.media().play(url, VLC_OPTIONS));
    }

    @Override
    public void pause() {
        checkInitialized();

        timer.stop();
        invokeOnVlc(() -> mediaPlayer.controls().pause());
    }

    @Override
    public void resume() {
        checkInitialized();

        timer.start();
        invokeOnVlc(() -> mediaPlayer.controls().play());
    }

    @Override
    public void seek(long time) {
        checkInitialized();

        invokeOnVlc(() -> mediaPlayer.controls().setTime(time));
    }

    @Override
    public void stop() {
        checkInitialized();

        invokeOnVlc(() -> mediaPlayer.controls().stop());
        surface.reset();
        timer.stop();
        reset();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing VLC player");

        try {
            this.mediaPlayer.videoSurface().set(surface);

            initialized = true;
            log.trace("VLC player initialization done");
        } catch (Exception ex) {
            log.error("Failed to initialize VLC player, " + ex.getMessage(), ex);
            setError(new VideoPlayerException(ex.getMessage(), ex));
        }
    }

    //endregion

    //region Functions

    @Override
    protected void initialize() {
        super.initialize();
        initializeListeners();
    }

    private void initializeListeners() {
        // if the time is being changed, make sure the animation drawer is running
        timeProperty().addListener((observable, oldValue, newValue) -> timer.start());
    }

    private void invokeOnVlc(Runnable runnable) {
        mediaPlayer.submit(() -> {
            try {
                runnable.run();
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
                setError(new VideoPlayerException(ex.getMessage(), ex));
            }
        });
    }

    //endregion
}

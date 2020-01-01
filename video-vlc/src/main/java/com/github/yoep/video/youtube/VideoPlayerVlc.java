package com.github.yoep.video.youtube;

import com.github.yoep.video.adapter.state.PlayerState;
import com.github.yoep.video.youtube.callback.FXBufferFormatCallback;
import com.github.yoep.video.youtube.callback.FXCallbackVideoSurface;
import com.github.yoep.video.youtube.callback.FXRenderCallback;
import javafx.scene.canvas.Canvas;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter;
import uk.co.caprica.vlcj.player.embedded.EmbeddedMediaPlayer;

@Slf4j
public class VideoPlayerVlc extends VideoPlayerYoutube {
    private final Canvas canvas = new Canvas();

    private final FXCallbackVideoSurface surface;
    private final MediaPlayerFactory mediaPlayerFactory;
    private final EmbeddedMediaPlayer mediaPlayer;
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

    //region VideoPlayer

    @Override
    public void initialize(Pane videoPane) {
        super.initialize(videoPane);
        init(videoPane);
    }

    @Override
    public void dispose() {
        super.dispose();

        stop();
        mediaPlayer.release();
        mediaPlayerFactory.release();
    }

    @Override
    public void play(String url) {
        super.play(url);

        if (!isYoutubeUrl(url)) {
            hide();

            timer.start();
            mediaPlayer.submit(() -> mediaPlayer.media().play(url));
        }
    }

    @Override
    public void pause() {
        super.pause();

        if (isYoutubePlayerActive())
            return;

        timer.stop();
        mediaPlayer.submit(() -> mediaPlayer.controls().pause());
    }

    @Override
    public void resume() {
        super.resume();

        if (isYoutubePlayerActive())
            return;

        timer.start();
        mediaPlayer.submit(() -> mediaPlayer.controls().play());
    }

    @Override
    public void seek(long time) {
        super.seek(time);

        if (isYoutubePlayerActive())
            return;

        mediaPlayer.submit(() -> mediaPlayer.controls().setTime(time));
    }

    @Override
    public void stop() {
        super.stop();

        if (isYoutubePlayerActive())
            return;

        mediaPlayer.submit(() -> mediaPlayer.controls().stop());
        timer.stop();
        surface.reset();
    }

    //endregion

    //region Functions

    private void initialize() {
        initializeEvents();
    }

    private void initializeEvents() {
        mediaPlayer.events().addMediaPlayerEventListener(new MediaPlayerEventAdapter() {
            @Override
            public void playing(MediaPlayer mediaPlayer) {
                setPlayerState(PlayerState.PLAYING);
            }

            @Override
            public void paused(MediaPlayer mediaPlayer) {
                setPlayerState(PlayerState.PAUSED);
            }

            @Override
            public void stopped(MediaPlayer mediaPlayer) {
                setPlayerState(PlayerState.STOPPED);
            }

            @Override
            public void finished(MediaPlayer mediaPlayer) {
                setPlayerState(PlayerState.FINISHED);
            }

            @Override
            public void error(MediaPlayer mediaPlayer) {
                log.warn("Media player went into error state");
                setPlayerState(PlayerState.ERROR);
            }

            @Override
            public void timeChanged(MediaPlayer mediaPlayer, long newTime) {
                setTime(newTime);
            }

            @Override
            public void lengthChanged(MediaPlayer mediaPlayer, long newLength) {
                setDuration(newLength);
            }
        });
    }

    private void init(Pane videoPane) {
        Assert.notNull(videoPane, "videoPane cannot be null");

        this.canvas.widthProperty().bind(videoPane.widthProperty());
        this.canvas.heightProperty().bind(videoPane.heightProperty());
        videoPane.getChildren().add(this.canvas);
        this.mediaPlayer.videoSurface().set(surface);
    }

    //endregion
}

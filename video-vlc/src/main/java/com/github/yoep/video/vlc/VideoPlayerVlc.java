package com.github.yoep.video.vlc;

import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.state.PlayerState;
import com.github.yoep.video.vlc.callback.FXBufferFormatCallback;
import com.github.yoep.video.vlc.callback.FXCallbackVideoSurface;
import com.github.yoep.video.vlc.callback.FXRenderCallback;
import com.github.yoep.video.youtube.VideoPlayerYoutube;
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

    private Throwable error;

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
    public Throwable getError() {
        return error != null ? error : super.getError();
    }

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
            invokeOnVlc(() -> mediaPlayer.media().play(url));
        }
    }

    @Override
    public void pause() {
        super.pause();

        if (isYoutubePlayerActive())
            return;

        timer.stop();
        invokeOnVlc(() -> mediaPlayer.controls().pause());
    }

    @Override
    public void resume() {
        super.resume();

        if (isYoutubePlayerActive())
            return;

        timer.start();
        invokeOnVlc(() -> mediaPlayer.controls().play());
    }

    @Override
    public void seek(long time) {
        super.seek(time);

        if (isYoutubePlayerActive())
            return;

        invokeOnVlc(() -> mediaPlayer.controls().setTime(time));
    }

    @Override
    public void stop() {
        super.stop();

        if (isYoutubePlayerActive())
            return;

        invokeOnVlc(() -> mediaPlayer.controls().stop());
        timer.stop();
        surface.reset();
        reset();
    }

    //endregion

    //region Functions


    @Override
    protected void reset() {
        super.reset();
        error = null;
    }

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
            public void buffering(MediaPlayer mediaPlayer, float newCache) {
                log.trace("VLC buffer is now {}%", newCache);
                if (newCache < 100) {
                    setPlayerState(PlayerState.BUFFERING);
                } else {
                    setPlayerState(PlayerState.PLAYING);
                }
            }

            @Override
            public void error(MediaPlayer mediaPlayer) {
                setError(new VideoPlayerException("VLC media player went into error state"));
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

    private void setError(Throwable throwable) {
        this.error = throwable;
        setPlayerState(PlayerState.ERROR);
    }

    //endregion
}

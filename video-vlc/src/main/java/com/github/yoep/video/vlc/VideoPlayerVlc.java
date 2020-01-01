package com.github.yoep.video.vlc;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import com.github.yoep.video.vlc.callback.FXBufferFormatCallback;
import com.github.yoep.video.vlc.callback.FXCallbackVideoSurface;
import com.github.yoep.video.vlc.callback.FXRenderCallback;
import javafx.beans.property.LongProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleLongProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.canvas.Canvas;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter;
import uk.co.caprica.vlcj.player.embedded.EmbeddedMediaPlayer;

@Slf4j
public class VideoPlayerVlc implements VideoPlayer {
    private final Canvas canvas = new Canvas();
    private final ObjectProperty<PlayerState> playerState = new SimpleObjectProperty<>(this, PLAYER_STATE_PROPERTY, PlayerState.UNKNOWN);
    private final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY);
    private final LongProperty length = new SimpleLongProperty(this, DURATION_PROPERTY);

    private final FXCallbackVideoSurface surface;
    private final MediaPlayerFactory mediaPlayerFactory;
    private final EmbeddedMediaPlayer mediaPlayer;
    private final VideoAnimationTimer timer;

    private boolean initialized;

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

    //region Properties

    @Override
    public PlayerState getPlayerState() {
        return playerState.get();
    }

    @Override
    public ObjectProperty<PlayerState> playerStateProperty() {
        return playerState;
    }

    /**
     * Set the state of the video player.
     *
     * @param playerState The new state of the video player.
     */
    private void setPlayerState(PlayerState playerState) {
        this.playerState.set(playerState);
    }

    @Override
    public long getTime() {
        return time.get();
    }

    @Override
    public LongProperty timeProperty() {
        return time;
    }

    /**
     * Set the current time of the media playback.
     *
     * @param time The time of the playback.
     */
    private void setTime(long time) {
        this.time.set(time);
    }

    @Override
    public long getDuration() {
        return length.get();
    }

    @Override
    public LongProperty durationProperty() {
        return length;
    }

    /**
     * Set the length of the current media playback.
     *
     * @param length The length in milliseconds.
     */
    private void setLength(long length) {
        this.length.set(length);
    }

    //endregion

    //region Getters & Setters

    @Override
    public boolean isInitialized() {
        return initialized;
    }

    //endregion

    //region VideoPlayer

    @Override
    public void initialize(Pane videoPane) {
        init(videoPane);
    }

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
        mediaPlayer.submit(() -> mediaPlayer.media().play(url));
    }

    @Override
    public void pause() {
        checkInitialized();

        timer.stop();
        mediaPlayer.submit(() -> mediaPlayer.controls().pause());
    }

    @Override
    public void resume() {
        checkInitialized();

        timer.start();
        mediaPlayer.submit(() -> mediaPlayer.controls().play());
    }

    @Override
    public void seek(long time) {
        checkInitialized();

        mediaPlayer.submit(() -> mediaPlayer.controls().setTime(time));
    }

    @Override
    public void stop() {
        timer.stop();

        mediaPlayer.submit(() -> mediaPlayer.controls().stop());
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
                setLength(newLength);
            }
        });
    }

    private void init(Pane videoPane) {
        Assert.notNull(videoPane, "videoPane cannot be null");

        this.canvas.widthProperty().bind(videoPane.widthProperty());
        this.canvas.heightProperty().bind(videoPane.heightProperty());
        videoPane.getChildren().add(this.canvas);
        this.mediaPlayer.videoSurface().set(surface);
        this.initialized = true;
    }

    private void checkInitialized() {
        if (!initialized)
            throw new VideoPlayerNotInitializedException(this);
    }

    //endregion
}

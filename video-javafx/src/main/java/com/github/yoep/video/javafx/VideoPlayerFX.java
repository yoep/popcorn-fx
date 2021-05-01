package com.github.yoep.video.javafx;

import com.github.yoep.video.adapter.AbstractVideoPlayer;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.listeners.VideoListener;
import com.github.yoep.video.adapter.state.VideoState;
import javafx.application.Platform;
import javafx.scene.Node;
import javafx.scene.layout.StackPane;
import javafx.scene.media.Media;
import javafx.scene.media.MediaException;
import javafx.scene.media.MediaPlayer;
import javafx.scene.media.MediaView;
import javafx.util.Duration;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;
import org.springframework.util.StringUtils;

import javax.annotation.PostConstruct;
import java.io.File;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = true)
public class VideoPlayerFX extends AbstractVideoPlayer implements VideoPlayer {
    private MediaView mediaView;
    private MediaPlayer mediaPlayer;

    private final StackPane stackPane = new StackPane();

    private Throwable error;
    private boolean initialized;

    //region Getters

    @Override
    public boolean supports(String url) {
        return !StringUtils.isEmpty(url);
    }

    @Override
    public boolean isInitialized() {
        return initialized;
    }

    @Override
    public Throwable getError() {
        return error;
    }

    @Override
    public Node getVideoSurface() {
        return stackPane;
    }

    //endregion

    //region VideoPlayer

    @Override
    public void dispose() {
        if (mediaPlayer != null)
            mediaPlayer.dispose();

        mediaView = null;
        mediaPlayer = null;
    }

    @Override
    public void addListener(VideoListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(VideoListener listener) {
        listeners.remove(listener);
    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {
        checkInitialized();

        try {
            mediaPlayer = new MediaPlayer(new Media(url));
            initializeMediaPlayerEvents();
            mediaView.setMediaPlayer(mediaPlayer);
            mediaPlayer.play();
        } catch (Exception ex) {
            setError(new VideoPlayerException("JavaFX video playback failed, " + ex.getMessage(), ex));
        }
    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {
        checkInitialized();

        mediaPlayer.pause();
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        checkInitialized();

        mediaPlayer.play();
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {
        checkInitialized();

        mediaPlayer.seek(Duration.millis(time));
    }

    @Override
    public void stop() {
        checkInitialized();

        mediaPlayer.stop();
        mediaPlayer = null;
        reset();
    }

    @Override
    public boolean supportsNativeSubtitleFile() {
        return false;
    }

    @Override
    public void subtitleFile(File file) {
        throw new UnsupportedOperationException("Subtitle file is not supported within the JavaFX player");
    }

    @Override
    public void subtitleDelay(long delay) {
        throw new UnsupportedOperationException("Subtitle delay is not supported within the JavaFX player");
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing JavaFX player");
        Platform.runLater(() -> {
            try {
                mediaView = new MediaView();

                mediaView.setPreserveRatio(true);
                mediaView.fitWidthProperty().bind(stackPane.widthProperty());
                mediaView.fitHeightProperty().bind(stackPane.heightProperty());

                stackPane.getChildren().add(mediaView);
                initialized = true;
                log.trace("JavaFX player initialization done");
            } catch (Exception ex) {
                log.error("Failed to initialize JavaFX player, " + ex.getMessage(), ex);
                setError(new VideoPlayerException(ex.getMessage(), ex));
            }
        });
    }

    //endregion

    //region Functions

    private void reset() {
        error = null;

        setTime(0L);
        setDuration(0L);
    }

    private void initializeMediaPlayerEvents() {
        if (mediaPlayer == null)
            return;

        setTime((long) mediaPlayer.getCurrentTime().toMillis());
        setDuration((long) mediaPlayer.getTotalDuration().toMillis());
        setVideoState(convertStatus(mediaPlayer.getStatus()));

        mediaPlayer.currentTimeProperty().addListener((observable, oldValue, newValue) -> setTime((long) newValue.toMillis()));
        mediaPlayer.totalDurationProperty().addListener((observable, oldValue, newValue) -> setDuration((long) newValue.toMillis()));
        mediaPlayer.statusProperty().addListener((observable, oldValue, newValue) -> setVideoState(convertStatus(newValue)));
        mediaPlayer.setOnEndOfMedia(() -> setVideoState(VideoState.FINISHED));
        mediaPlayer.setOnError(this::onError);
    }

    private VideoState convertStatus(MediaPlayer.Status status) {
        switch (status) {
            case PLAYING:
                return VideoState.PLAYING;
            case PAUSED:
                return VideoState.PAUSED;
            case STOPPED:
                return VideoState.STOPPED;
            case UNKNOWN:
            default:
                return VideoState.UNKNOWN;
        }
    }

    private void onError() {
        MediaException error = mediaPlayer.getError();
        log.error("JavaFX player encountered an error, " + error.getMessage(), error);

        setError(error);
    }

    private void setError(Throwable throwable) {
        this.error = throwable;
        setVideoState(VideoState.ERROR);
    }

    private void checkInitialized() {
        if (!initialized)
            throw new VideoPlayerNotInitializedException(this);
    }

    //endregion
}

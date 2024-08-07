package com.github.yoep.video.javafx;

import com.github.yoep.popcorn.backend.adapters.video.AbstractVideoPlayer;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerException;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerNotInitializedException;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
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

import java.io.File;
import java.util.Objects;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = true)
public class VideoPlayerFX extends AbstractVideoPlayer implements VideoPlayback {
    static final String NAME = "FX";
    static final String DESCRIPTION = "Video playback which uses the JavaFX video playback device. This is in most cases used as a fallback backend.";

    private MediaView mediaView;
    private MediaPlayer mediaPlayer;

    private final StackPane stackPane = new StackPane();

    private Throwable error;
    private boolean initialized;

    public VideoPlayerFX() {
        init();
    }

    //region Getters

    @Override
    public String getName() {
        return NAME;
    }

    @Override
    public String getDescription() {
        return DESCRIPTION;
    }

    @Override
    public boolean supports(String url) {
        return url != null && !url.isBlank();
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
        Objects.requireNonNull(listener, "listener cannot be null");
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
    public void volume(int volume) {
        checkInitialized();

        mediaPlayer.setVolume((double) volume / 100);
    }

    @Override
    public int getVolume() {
        checkInitialized();

        return (int) (mediaPlayer.getVolume() * 100);
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
                setVideoState(VideoState.READY);
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

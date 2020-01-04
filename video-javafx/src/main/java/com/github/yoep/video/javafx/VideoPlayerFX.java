package com.github.yoep.video.javafx;

import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import com.github.yoep.video.youtube.VideoPlayerYoutube;
import javafx.application.Platform;
import javafx.scene.layout.Pane;
import javafx.scene.media.Media;
import javafx.scene.media.MediaException;
import javafx.scene.media.MediaPlayer;
import javafx.scene.media.MediaView;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.net.URI;

@Slf4j
public class VideoPlayerFX extends VideoPlayerYoutube {
    private MediaView mediaView;
    private MediaPlayer mediaPlayer;

    private Throwable error;

    //region VideoPlayer

    @Override
    public Throwable getError() {
        return error != null ? error : super.getError();
    }

    @Override
    public void initialize(Pane videoPane) {
        super.initialize(videoPane);

        initializeMediaView(videoPane);
    }

    @Override
    public void dispose() {
        mediaPlayer.dispose();
        mediaView = null;
        mediaPlayer = null;
    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {
        super.play(url);

        if (!isYoutubeUrl(url)) {
            if (mediaView == null) {
                log.error("Unable to play the given url, media view failed to initialize");
                return;
            }

            hide();

            try {
                URI uri = new File(url).toURI();
                Media media = new Media(uri.toString());

                mediaPlayer = new MediaPlayer(media);
                initializeMediaPlayerEvents();
                mediaView.setMediaPlayer(mediaPlayer);
                mediaPlayer.play();
            } catch (Exception ex) {
                setError(new VideoPlayerException("JavaFX video playback failed, " + ex.getMessage(), ex));
            }
        }
    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {
        super.pause();

        if (isYoutubePlayerActive())
            return;

        mediaPlayer.pause();
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        super.resume();

        if (isYoutubePlayerActive())
            return;

        mediaPlayer.play();
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {
        super.seek(time);

        if (isYoutubePlayerActive())
            return;

        mediaPlayer.seek(Duration.millis(time));
    }

    @Override
    public void stop() {
        super.stop();

        if (isYoutubePlayerActive() || mediaPlayer == null)
            return;

        mediaPlayer.stop();
        mediaPlayer = null;
        reset();
    }

    //endregion

    //region Functions

    @Override
    protected void reset() {
        super.reset();
        error = null;
    }

    private void initializeMediaView(Pane videoPane) {
        Platform.runLater(() -> {
            try {
                mediaView = new MediaView();

                mediaView.fitHeightProperty().bind(videoPane.heightProperty());
                mediaView.fitWidthProperty().bind(videoPane.widthProperty());
                mediaView.setPreserveRatio(true);

                videoPane.getChildren().add(mediaView);
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
                setError(new VideoPlayerException(ex.getMessage(), ex));
            }
        });
    }

    private void initializeMediaPlayerEvents() {
        if (mediaPlayer == null)
            return;

        setTime((long) mediaPlayer.getCurrentTime().toMillis());
        setDuration((long) mediaPlayer.getTotalDuration().toMillis());
        setPlayerState(convertStatus(mediaPlayer.getStatus()));

        mediaPlayer.currentTimeProperty().addListener((observable, oldValue, newValue) -> setTime((long) newValue.toMillis()));
        mediaPlayer.totalDurationProperty().addListener((observable, oldValue, newValue) -> setDuration((long) newValue.toMillis()));
        mediaPlayer.statusProperty().addListener((observable, oldValue, newValue) -> setPlayerState(convertStatus(newValue)));
        mediaPlayer.setOnEndOfMedia(() -> setPlayerState(PlayerState.FINISHED));
        mediaPlayer.setOnError(this::onError);
    }

    private PlayerState convertStatus(MediaPlayer.Status status) {
        switch (status) {
            case PLAYING:
                return PlayerState.PLAYING;
            case PAUSED:
                return PlayerState.PAUSED;
            case STOPPED:
                return PlayerState.STOPPED;
            case UNKNOWN:
            default:
                return PlayerState.UNKNOWN;
        }
    }

    private void onError() {
        MediaException error = mediaPlayer.getError();
        log.error("JavaFX player encountered an error, " + error.getMessage(), error);

        setError(error);
    }

    private void setError(Throwable throwable) {
        this.error = throwable;
        setPlayerState(PlayerState.ERROR);
    }

    //endregion
}

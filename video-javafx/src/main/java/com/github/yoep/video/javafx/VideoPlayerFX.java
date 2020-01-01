package com.github.yoep.video.javafx;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.application.Platform;
import javafx.beans.property.LongProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleLongProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.layout.Pane;
import javafx.scene.media.Media;
import javafx.scene.media.MediaException;
import javafx.scene.media.MediaPlayer;
import javafx.scene.media.MediaView;
import javafx.scene.web.WebEngine;
import javafx.scene.web.WebView;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;
import netscape.javascript.JSObject;
import org.apache.commons.io.IOUtils;
import org.springframework.core.io.ClassPathResource;

import java.io.IOException;
import java.nio.charset.Charset;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Slf4j
public class VideoPlayerFX implements VideoPlayer {
    private static final Pattern VIDEO_ID_PATTERN = Pattern.compile("watch\\?v=([^#&?]*)");

    private final ObjectProperty<PlayerState> playerState = new SimpleObjectProperty<>(this, PLAYER_STATE_PROPERTY, PlayerState.UNKNOWN);
    private final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY);
    private final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY);

    private MediaView mediaView;
    private MediaPlayer mediaPlayer;
    private WebView webView;
    private boolean initialized;

    //region Properties

    @Override
    public PlayerState getPlayerState() {
        return playerState.get();
    }

    @Override
    public ObjectProperty<PlayerState> playerStateProperty() {
        return playerState;
    }

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

    private void setTime(long time) {
        this.time.set(time);
    }

    @Override
    public long getDuration() {
        return duration.get();
    }

    @Override
    public LongProperty durationProperty() {
        return duration;
    }

    private void setDuration(long duration) {
        this.duration.set(duration);
    }


    //endregion

    //region Getters

    @Override
    public boolean isInitialized() {
        return initialized;
    }

    //endregion

    //region VideoPlayer

    @Override
    public void initialize(Pane videoPane) {
        Platform.runLater(() -> {
            mediaView = new MediaView();
            webView = new WebView();

            webView.prefWidthProperty().bind(videoPane.widthProperty());
            webView.prefHeightProperty().bind(videoPane.heightProperty());
            initializeWebviewEvents();

            videoPane.getChildren().add(webView);
            videoPane.getChildren().add(mediaView);

            initialized = true;
        });
    }

    @Override
    public void dispose() {
        mediaPlayer.dispose();
        mediaView = null;
    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {
        checkInitialized();

        if (isYoutubeUrl(url)) {
            playYoutubeUrl(url);
        } else {
            switchView(false);
            mediaPlayer = new MediaPlayer(new Media(url));
            initializeMediaPlayerEvents();
            mediaPlayer.play();
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
        if (mediaPlayer == null)
            return;

        mediaPlayer.stop();
    }

    //endregion

    //region Functions

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

    private void initializeWebviewEvents() {
        WebEngine engine = webView.getEngine();

        engine.documentProperty().addListener((observable, oldValue, newValue) -> {
            JSObject window = (JSObject) engine.executeScript("window");
            //            window.setMember("OpenDoc", );
        });
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

    private void checkInitialized() {
        if (!initialized)
            throw new VideoPlayerNotInitializedException(this);
    }

    private String getVideoId(String url) {
        Matcher matcher = VIDEO_ID_PATTERN.matcher(url);

        if (matcher.find()) {
            return matcher.group(1);
        } else {
            throw new VideoPlayerException("Failed to play youtube url, unable to retrieve video id");
        }
    }

    private boolean isYoutubeUrl(String url) {
        return url.toLowerCase().contains("youtu");
    }

    private void switchView(boolean isWebViewVisible) {
        Platform.runLater(() -> {
            mediaView.setVisible(!isWebViewVisible);
            webView.setVisible(isWebViewVisible);
        });
    }

    private void playYoutubeUrl(String url) {
        switchView(true);
        ClassPathResource resource = new ClassPathResource("embed_youtube.html");

        try {
            String content = IOUtils.toString(resource.getInputStream(), Charset.defaultCharset()).replace("[[VIDEO_ID]]", getVideoId(url));

            Platform.runLater(() -> {
                webView.getEngine().setJavaScriptEnabled(true);
                webView.getEngine().loadContent(content);
            });
        } catch (IOException e) {
            throw new VideoPlayerException(e.getMessage(), e);
        }
    }

    private void onError() {
        MediaException error = mediaPlayer.getError();
        log.error("JavaFX player encountered an error, " + error.getMessage(), error);

        setPlayerState(PlayerState.ERROR);
    }

    //endregion
}

package com.github.yoep.video.youtube;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.application.Platform;
import javafx.beans.property.LongProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleLongProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.concurrent.Worker;
import javafx.scene.layout.Pane;
import javafx.scene.web.WebEngine;
import javafx.scene.web.WebView;
import lombok.extern.slf4j.Slf4j;
import netscape.javascript.JSObject;
import org.apache.commons.io.IOUtils;
import org.springframework.core.io.ClassPathResource;
import org.springframework.util.StringUtils;

import java.io.IOException;
import java.nio.charset.Charset;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Slf4j
public class VideoPlayerYoutube implements VideoPlayer {
    private static final Pattern VIDEO_ID_PATTERN = Pattern.compile("watch\\?v=([^#&?]*)");
    private static final String YOUTUBE_URL_INDICATOR = "youtu";

    protected final ObjectProperty<PlayerState> playerState = new SimpleObjectProperty<>(this, PLAYER_STATE_PROPERTY, PlayerState.UNKNOWN);
    protected final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY);
    protected final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY);
    private final YoutubePlayerBridge playerBridge = new YoutubePlayerBridge();

    private Pane videoPane;
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

    protected void setPlayerState(PlayerState playerState) {
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

    protected void setTime(long time) {
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

    protected void setDuration(long duration) {
        this.duration.set(duration);
    }

    //endregion

    //region Getters

    @Override
    public boolean isInitialized() {
        return initialized;
    }

    /**
     * Check if the given url is q youtube url.
     *
     * @param url The url to verify.
     * @return Returns true if the url is a youtube url, else false.
     */
    public boolean isYoutubeUrl(String url) {
        if (StringUtils.isEmpty(url))
            return false;

        return url.toLowerCase().contains(YOUTUBE_URL_INDICATOR);
    }

    /**
     * Verify if the youtube player is active (shown).
     *
     * @return Returns true if the player is shown, else false.
     */
    public boolean isYoutubePlayerActive() {
        return videoPane.getChildren().contains(webView);
    }

    //endregion

    //region VideoPlayer

    @Override
    public void initialize(Pane videoPane) {
        this.videoPane = videoPane;

        Platform.runLater(() -> {
            webView = new WebView();

            webView.prefWidthProperty().bind(videoPane.widthProperty());
            webView.prefHeightProperty().bind(videoPane.heightProperty());
            initializeWebviewEvents();

            videoPane.getChildren().add(webView);
            initialized = true;
        });
    }

    @Override
    public void dispose() {
        webView = null;
    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {
        checkInitialized();

        if (!isYoutubeUrl(url))
            return;

        playYoutubeUrl(url);
    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {
        checkInitialized();

        if (!isYoutubePlayerActive())
            return;

        Platform.runLater(() -> getEngine().executeScript("pause()"));
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        checkInitialized();

        if (!isYoutubePlayerActive())
            return;

        Platform.runLater(() -> getEngine().executeScript("resume()"));
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {
        checkInitialized();

        if (!isYoutubePlayerActive())
            return;

        Platform.runLater(() -> getEngine().executeScript("seek(" + time + ")"));
    }

    @Override
    public void stop() {
        checkInitialized();

        if (!isYoutubePlayerActive())
            return;

        Platform.runLater(() -> getEngine().executeScript("stop()"));
    }

    //endregion

    //region Methods

    /**
     * Show the youtube player.
     */
    public void show() {
        if (isYoutubePlayerActive())
            return;

        Platform.runLater(() -> videoPane.getChildren().add(webView));
    }

    /**
     * Hide the youtube player.
     */
    public void hide() {
        Platform.runLater(() -> videoPane.getChildren().remove(webView));
    }

    //endregion

    //region Functions

    private void checkInitialized() {
        if (!initialized)
            throw new VideoPlayerNotInitializedException(this);
    }

    private void initializeWebviewEvents() {
        WebEngine engine = getEngine();
        ClassPathResource resource = new ClassPathResource("embed_youtube.html");

        engine.setJavaScriptEnabled(true);
        engine.getLoadWorker().stateProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue == Worker.State.RUNNING) {
                JSObject window = (JSObject) engine.executeScript("window");
                window.setMember("VPY", playerBridge);
            }
        });

        try {
            String content = IOUtils.toString(resource.getInputStream(), Charset.defaultCharset());

            Platform.runLater(() -> getEngine().loadContent(content));
        } catch (IOException e) {
            throw new VideoPlayerException(e.getMessage(), e);
        }
    }

    private void playYoutubeUrl(String url) {
        show();
        String videoId = getVideoId(url);

        Platform.runLater(() -> getEngine().executeScript("window.play('" + videoId + "');"));
    }

    private String getVideoId(String url) {
        Matcher matcher = VIDEO_ID_PATTERN.matcher(url);

        if (matcher.find()) {
            return matcher.group(1);
        } else {
            throw new VideoPlayerException("Failed to play youtube url, unable to retrieve video id");
        }
    }

    private WebEngine getEngine() {
        return webView.getEngine();
    }

    //endregion

    public class YoutubePlayerBridge {
        public void state(String state) {
            switch (state) {
                case "playing":
                    setPlayerState(PlayerState.PLAYING);
                    break;
                case "paused":
                    setPlayerState(PlayerState.PAUSED);
                    break;
                case "ended":
                    setPlayerState(PlayerState.STOPPED);
                    break;
                default:
                    setPlayerState(PlayerState.UNKNOWN);
                    break;
            }
        }

        public void time(long time) {
            setTime(time * 1000);
        }

        public void duration(long time) {
            setDuration(time * 1000);
        }

        public void log(String message) {
            log.debug("[WebView] " + message);
        }
    }
}

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
import javafx.scene.Node;
import javafx.scene.web.WebEngine;
import javafx.scene.web.WebView;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import netscape.javascript.JSObject;
import org.apache.commons.io.IOUtils;
import org.springframework.core.io.ClassPathResource;
import org.springframework.util.StringUtils;

import javax.annotation.PostConstruct;
import java.io.IOException;
import java.nio.charset.Charset;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Slf4j
@ToString
@EqualsAndHashCode
public class VideoPlayerYoutube implements VideoPlayer {
    private static final Pattern VIDEO_ID_PATTERN = Pattern.compile("watch\\?v=([^#&?]*)");
    private static final String YOUTUBE_URL_INDICATOR = "youtu";

    private final ObjectProperty<PlayerState> playerState = new SimpleObjectProperty<>(this, PLAYER_STATE_PROPERTY, PlayerState.UNKNOWN);
    private final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY);
    private final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY);
    private final YoutubePlayerBridge playerBridge = new YoutubePlayerBridge();

    private WebView webView;
    private boolean initialized;
    private boolean playerReady;

    private Throwable error;

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
    public boolean supports(String url) {
        if (StringUtils.isEmpty(url))
            return false;

        return url.toLowerCase().contains(YOUTUBE_URL_INDICATOR);
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
        checkInitialized();

        return webView;
    }

    //endregion

    //region VideoPlayer

    @Override
    public void dispose() {
        webView = null;
    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {
        checkInitialized();

        playYoutubeUrl(url);
    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {
        checkInitialized();

        Platform.runLater(() -> getEngine().executeScript("pause()"));
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        checkInitialized();

        Platform.runLater(() -> getEngine().executeScript("resume()"));
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {
        checkInitialized();

        Platform.runLater(() -> getEngine().executeScript("seek(" + time + ")"));
    }

    @Override
    public void stop() {
        checkInitialized();

        Platform.runLater(() -> {
            getEngine().executeScript("stop()");
            reset();
        });
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing youtube player");
        Platform.runLater(() -> {
            try {
                webView = new WebView();

                initializeWebviewEvents();

                initialized = true;
                log.trace("Youtube player initialization done");
            } catch (Exception ex) {
                log.error("Failed to initialize youtube player, " + ex.getMessage(), ex);
                setError(new VideoPlayerException(ex.getMessage(), ex));
            }
        });
    }

    //endregion

    //region Functions

    /**
     * Reset the video player information.
     * This will reset the last error that occurred and reset the time & duration so the event are correctly fired on next video play.
     * <p>
     * (Fixes the duration event not firing if the video has the same duration as the last video)
     */
    protected void reset() {
        error = null;

        setTime(0);
        setDuration(0);
    }

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
        String videoId = getVideoId(url);

        new Thread(() -> {
            try {
                while (!playerReady) {
                    Thread.sleep(100);
                }

                Platform.runLater(() -> getEngine().executeScript("window.play('" + videoId + "');"));
            } catch (InterruptedException ex) {
                log.error("Unexpectedly quit of wait for webview worker monitor", ex);
            }
        }, "YoutubePlayer-monitor").start();
    }

    private String getVideoId(String url) {
        Matcher matcher = VIDEO_ID_PATTERN.matcher(url);

        if (matcher.find()) {
            return matcher.group(1);
        } else {
            throw new VideoPlayerException("Failed to play youtube url, unable to retrieve video id");
        }
    }

    private void setError(Throwable throwable) {
        this.error = throwable;
        setPlayerState(PlayerState.ERROR);
    }

    private WebEngine getEngine() {
        return webView.getEngine();
    }

    //endregion

    @SuppressWarnings("unused")
    public class YoutubePlayerBridge {
        public void ready() {
            playerReady = true;
        }

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
            log.trace("[WebView] " + message);
        }

        public void error(String code) {
            setError(new VideoPlayerException("Youtube Player encountered an issue, error code " + code));
        }
    }
}

package com.github.yoep.video.youtube;

import com.github.yoep.popcorn.backend.adapters.video.AbstractVideoPlayer;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerException;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerNotInitializedException;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import javafx.application.Platform;
import javafx.concurrent.Worker;
import javafx.scene.Node;
import javafx.scene.web.WebEngine;
import javafx.scene.web.WebView;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import netscape.javascript.JSException;
import netscape.javascript.JSObject;
import org.apache.commons.io.IOUtils;

import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.text.MessageFormat;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.function.Consumer;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Slf4j
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
public class VideoPlayerYoutube extends AbstractVideoPlayer implements VideoPlayback {
    static final Pattern VIDEO_ID_PATTERN = Pattern.compile("watch\\?v=([^#&?]*)");
    static final String YOUTUBE_URL_INDICATOR = "youtu";
    static final String NAME = "Youtube";
    static final String DESCRIPTION = "Video playback backend which uses a dedicated web view for playback of Youtube videos.";
    private static final int BRIDGE_TIMEOUT = 3000;

    private final YoutubePlayerBridge playerBridge = new YoutubePlayerBridge();

    private WebView webView;
    private boolean initialized;
    private boolean playerReady;

    private Throwable error;

    public VideoPlayerYoutube() {
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
        if (url == null || url.isBlank())
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

        playYoutubeUrl(url);
    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {
        checkInitialized();

        Platform.runLater(() -> invokeOnEngine(e -> e.executeScript("pause()")));
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        checkInitialized();

        Platform.runLater(() -> invokeOnEngine(e -> e.executeScript("resume()")));
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {
        checkInitialized();

        Platform.runLater(() -> {
            try {
                getEngine().executeScript("seek(" + time + ")");
            } catch (JSException ex) {
                var message = MessageFormat.format("Failed to seek youtube player, {0}", ex.getMessage());
                log.error(message, ex);
            }
        });
    }

    @Override
    public void stop() {
        if (initialized) {
            Platform.runLater(this::stopPlayer);
        }

        setVideoState(VideoState.STOPPED);
    }

    @Override
    public void volume(int volume) {
        checkInitialized();

        Platform.runLater(() -> invokeOnEngine(e -> e.executeScript("volume(" + volume + ")")));
    }

    @Override
    public int getVolume() {
        checkInitialized();
        var volume = new CompletableFuture<Integer>();

        Platform.runLater(() -> invokeOnEngine(e -> volume.complete((Integer) e.executeScript("getVolume()"))));

        try {
            return volume.get(2, TimeUnit.SECONDS);
        } catch (InterruptedException | TimeoutException | ExecutionException ex) {
            log.warn("Unable to retrieve youtube player volume, " + ex.getMessage(), ex);
            return 100;
        }
    }

    @Override
    public boolean supportsNativeSubtitleFile() {
        return false;
    }

    @Override
    public void subtitleFile(File file) {
        throw new UnsupportedOperationException("Subtitle file is not supported within the Youtube player");
    }

    @Override
    public void subtitleDelay(long delay) {
        throw new UnsupportedOperationException("Subtitle delay is not supported within the Youtube player");
    }

    //endregion

    //region PostConstruct

    private void init() {
        log.trace("Initializing youtube player");
        Platform.runLater(() -> {
            try {
                webView = new WebView();
                webView.setFocusTraversable(false);

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

        setTime(0L);
        setDuration(0L);
    }

    private void checkInitialized() {
        if (!initialized)
            throw new VideoPlayerNotInitializedException(this);
    }

    private void initializeWebviewEvents() {
        var engine = getEngine();

        try (var resource = VideoPlayerYoutube.class.getResourceAsStream("/embed_youtube.html")) {
            engine.setJavaScriptEnabled(true);
            engine.getLoadWorker().stateProperty().addListener((observable, oldValue, newValue) -> {
                if (newValue == Worker.State.RUNNING) {
                    JSObject window = (JSObject) engine.executeScript("window");
                    window.setMember("VPY", playerBridge);
                } else if (newValue == Worker.State.SUCCEEDED) {
                    JSObject window = (JSObject) engine.executeScript("window");
                    window.setMember("VPY", playerBridge);
                }
            });

            var content = IOUtils.toString(resource, Charset.defaultCharset());
            Platform.runLater(() -> getEngine().loadContent(content));
        } catch (IOException e) {
            throw new VideoPlayerException(e.getMessage(), e);
        }
    }

    private void playYoutubeUrl(String url) {
        String videoId = getVideoId(url);

        new Thread(() -> {
            try {
                if (waitForPlayerToBeReady()) {
                    Platform.runLater(() -> getEngine().executeScript("window.play('" + videoId + "');"));
                } else {
                    setError(new PlayerStateException("Youtube player state failed, player not ready"));
                    setVideoState(VideoState.ERROR);
                }
            } catch (InterruptedException ex) {
                log.error("Unexpectedly quit of wait for webview worker monitor", ex);
            }
        }, "YoutubePlayer-monitor").start();
    }

    private boolean waitForPlayerToBeReady() throws InterruptedException {
        var startTime = System.currentTimeMillis();

        while (shouldWaitForBridgePlayerToBeReady(startTime)) {
            setVideoState(VideoState.BUFFERING);
            Thread.onSpinWait();
        }

        return playerReady;
    }

    private boolean shouldWaitForBridgePlayerToBeReady(long startTime) {
        // if the player is not ready
        // wait for the bridge to communicate that it's ready to receive playback requests
        // unless it exceeds the timeout for which we allow the bridge to wait
        return !playerReady && System.currentTimeMillis() - startTime < BRIDGE_TIMEOUT;
    }

    private void stopPlayer() {
        if (playerReady) {
            stopEnginePlayer();
        }

        reset();
        setVideoState(VideoState.STOPPED);
    }

    private void stopEnginePlayer() {
        try {
            getEngine().executeScript("stop()");
        } catch (JSException ex) {
            log.error("Failed to stop youtube player, {}", ex.getMessage(), ex);
        }
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
        setVideoState(VideoState.ERROR);
    }

    private WebEngine getEngine() {
        return webView.getEngine();
    }

    private void invokeOnEngine(Consumer<WebEngine> action) {
        try {
            action.accept(getEngine());
        } catch (JSException ex) {
            log.error("Failed to invoke youtube engine command", ex);
            setVideoState(VideoState.ERROR);
        }
    }

    //endregion

    @SuppressWarnings("unused")
    public class YoutubePlayerBridge {
        public void ready() {
            playerReady = true;
            setVideoState(VideoState.READY);
        }

        public void state(String state) {
            switch (state) {
                case "playing" -> setVideoState(VideoState.PLAYING);
                case "paused" -> setVideoState(VideoState.PAUSED);
                case "ended" -> setVideoState(VideoState.STOPPED);
                case "buffering" -> setVideoState(VideoState.BUFFERING);
                default -> setVideoState(VideoState.UNKNOWN);
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

        public void logWarn(String message) {
            log.warn("[WebView] " + message);
        }

        public void logError(String message) {
            log.error("[WebView] " + message);
        }

        public void error(String code) {
            setError(new VideoPlayerException("Youtube Player encountered an issue, error code " + code));
        }
    }
}

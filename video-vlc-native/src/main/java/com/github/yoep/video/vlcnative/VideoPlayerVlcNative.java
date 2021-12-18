package com.github.yoep.video.vlcnative;

import com.github.yoep.popcorn.backend.adapters.video.AbstractVideoPlayer;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayer;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerException;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerNotInitializedException;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import com.github.yoep.video.vlcnative.player.PopcornPlayer;
import com.github.yoep.video.vlcnative.player.PopcornPlayerEventListener;
import com.github.yoep.video.vlcnative.player.PopcornPlayerState;
import javafx.scene.Node;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.io.File;

@Slf4j
public class VideoPlayerVlcNative extends AbstractVideoPlayer implements VideoPlayer {
    private static final Pane videoSurfaceTracker = new StackPane();

    private PopcornPlayer popcornPlayer;
    private boolean initialized;

    //region VideoPlayer

    @Override
    public boolean supports(String url) {
        // if the native player could be initialized
        // than use it for every playback
        return initialized;
    }

    @Override
    public boolean isInitialized() {
        return initialized;
    }

    @Override
    public Throwable getError() {
        return null;
    }

    @Override
    public Node getVideoSurface() {
        return videoSurfaceTracker;
    }

    @Override
    public void dispose() {
        if (popcornPlayer != null) {
            log.debug("Releasing the Popcorn Player");
            popcornPlayer.release();
            popcornPlayer = null;
        } else {
            log.trace("Popcorn Player has already been disposed");
        }
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

        // start the native player through JNA
        popcornPlayer.show();
        popcornPlayer.fullscreen(true);
        popcornPlayer.play(url);
    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {
        checkInitialized();
        popcornPlayer.pause();
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        checkInitialized();
        popcornPlayer.resume();
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {
        checkInitialized();
        popcornPlayer.seek(time);
    }

    @Override
    public void stop() {
        checkInitialized();
        popcornPlayer.stop();
        reset();
    }

    @Override
    public boolean supportsNativeSubtitleFile() {
        return true;
    }

    @Override
    public void subtitleFile(File file) {
        Assert.notNull(file, "file cannot be null");
        popcornPlayer.subtitleFile(file);
    }

    @Override
    public void subtitleDelay(long delay) {
        log.trace("Updating subtitle delay to {} milliseconds", delay);
        popcornPlayer.subtitleDelay(delay);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing popcorn native player");
        try {
            var level = getPlayerLogLevel();
            var args = new String[]{"PopcornPlayer", "-l", level};

            popcornPlayer = new PopcornPlayer(args);

            initializeListener();
            initialized = true;
            log.debug("Popcorn player has been initialized");
        } catch (Exception ex) {
            log.error("Failed to load the popcorn player library, " + ex.getMessage(), ex);
        }
    }

    private void initializeListener() {
        popcornPlayer.addListener(new PopcornPlayerEventListener() {
            @Override
            public void onStateChanged(PopcornPlayerState newState) {
                switch (newState) {
                    case PLAYING:
                        updateState(VideoState.PLAYING);
                        break;
                    case PAUSED:
                        updateState(VideoState.PAUSED);
                        break;
                    case BUFFERING:
                        updateState(VideoState.BUFFERING);
                        break;
                    case STOPPED:
                        updateState(VideoState.STOPPED);
                        break;
                    case UNKNOWN:
                        updateState(VideoState.UNKNOWN);
                        break;
                    default:
                        log.error("Received unknown popcorn player state " + newState);
                        break;
                }
            }

            @Override
            public void onTimeChanged(long newValue) {
                if (newValue >= 0) {
                    setTime(newValue);
                } else {
                    log.warn("Received invalid time value {}", newValue);
                }
            }

            @Override
            public void onDurationChanged(long newValue) {
                if (newValue >= 0) {
                    log.trace("Popcorn player duration changed to {}", newValue);
                    setDuration(newValue);
                } else {
                    log.warn("Received invalid duration value {}", newValue);
                }
            }
        });
    }

    //endregion

    //region Functions

    private void checkInitialized() {
        if (!initialized) {
            throw new VideoPlayerException("Popcorn player has not yet been initialized");
        }
    }

    private void reset() {
        setTime(0L);
        setDuration(0L);
    }

    private void updateState(VideoState newState) {
        log.debug("Popcorn player state changed to " + newState);
        setVideoState(newState);
    }

    private String getPlayerLogLevel() {
        if (log.isTraceEnabled()) {
            return "trace";
        } else if (log.isDebugEnabled()) {
            return "debug";
        }

        return "info";
    }

    //endregion
}

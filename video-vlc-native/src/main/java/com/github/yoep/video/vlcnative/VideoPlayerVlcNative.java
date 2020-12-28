package com.github.yoep.video.vlcnative;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import com.github.yoep.video.vlcnative.player.PopcornPlayer;
import com.github.yoep.video.vlcnative.player.PopcornPlayerEventListener;
import com.github.yoep.video.vlcnative.player.PopcornPlayerState;
import javafx.beans.property.*;
import javafx.scene.Node;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.io.File;

@Slf4j
public class VideoPlayerVlcNative implements VideoPlayer {
    private static final Pane videoSurfaceTracker = new StackPane();

    private final ObjectProperty<PlayerState> playerState = new SimpleObjectProperty<>(this, PLAYER_STATE_PROPERTY, PlayerState.UNKNOWN);
    private final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY);
    private final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY);

    private PopcornPlayer popcornPlayer;
    private boolean initialized;

    //region VideoPlayer

    @Override
    public PlayerState getPlayerState() {
        return playerState.get();
    }

    @Override
    public ReadOnlyObjectProperty<PlayerState> playerStateProperty() {
        return playerState;
    }

    @Override
    public long getTime() {
        return time.get();
    }

    @Override
    public ReadOnlyLongProperty timeProperty() {
        return time;
    }

    @Override
    public long getDuration() {
        return duration.get();
    }

    @Override
    public ReadOnlyLongProperty durationProperty() {
        return duration;
    }

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
                        updateState(PlayerState.PLAYING);
                        break;
                    case PAUSED:
                        updateState(PlayerState.PAUSED);
                        break;
                    case BUFFERING:
                        updateState(PlayerState.BUFFERING);
                        break;
                    case STOPPED:
                        updateState(PlayerState.STOPPED);
                        break;
                    case UNKNOWN:
                        updateState(PlayerState.UNKNOWN);
                        break;
                    default:
                        log.error("Received unknown popcorn player state " + newState);
                        break;
                }
            }

            @Override
            public void onTimeChanged(long newValue) {
                if (newValue >= 0) {
                    time.set(newValue);
                } else {
                    log.warn("Received invalid time value {}", newValue);
                }
            }

            @Override
            public void onDurationChanged(long newValue) {
                if (newValue >= 0) {
                    log.trace("Popcorn player duration changed to {}", newValue);
                    duration.setValue(newValue);
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
        time.set(0);
        duration.set(0);
    }

    private void updateState(PlayerState newState) {
        log.debug("Popcorn player state changed to " + newState);
        playerState.set(newState);
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

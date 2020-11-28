package com.github.yoep.video.vlcnative;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import com.github.yoep.video.vlcnative.player.PopcornPlayer;
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
            log.debug("Releasing the native Popcorn Player");
            popcornPlayer.release();
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
        playerState.set(PlayerState.PAUSED);
    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {
        checkInitialized();
        popcornPlayer.resume();
        playerState.set(PlayerState.PLAYING);
    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {

    }

    @Override
    public void stop() {
        checkInitialized();
        popcornPlayer.stop();
        playerState.set(PlayerState.STOPPED);
        reset();
    }

    @Override
    public boolean supportsNativeSubtitleFile() {
        return true;
    }

    @Override
    public void subtitleFile(File file) {
        Assert.notNull(file, "file cannot be null");
        log.trace("Adding subtitle file {} to the current playback", file.getAbsolutePath());
//        PopcornPlayerLib.popcorn_player_subtitle(popcornPlayer, file.getAbsolutePath());
    }

    @Override
    public void subtitleDelay(long delay) {
        log.trace("Updating subtitle delay to {} milliseconds", delay);
//        PopcornPlayerLib.popcorn_player_subtitle_delay(popcornPlayer, delay * 1000);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing VLC native player");
        try {
            var level = getPlayerLogLevel();
            var args = new String[]{"PopcornPlayer", "-l", level};

            popcornPlayer = new PopcornPlayer(args);
            initialized = true;
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    //endregion

    //region Functions

    private void checkInitialized() {
        if (!initialized) {
            throw new VideoPlayerException("VLC native player has not yet been initialized");
        }
    }

    private void reset() {
        time.set(0);
        duration.set(0);
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

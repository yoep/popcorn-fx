package com.github.yoep.video.vlc;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.beans.property.*;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.StringUtils;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter;

import java.io.File;

/**
 * Abstract implementation of the {@link VideoPlayer} adapter.
 *
 * @param <T> The media player instance that is used by the {@link AbstractVideoPlayer}.
 */
@Slf4j
abstract class AbstractVideoPlayer<T extends MediaPlayer> implements VideoPlayer {
    protected final ObjectProperty<PlayerState> playerState = new SimpleObjectProperty<>(this, PLAYER_STATE_PROPERTY, PlayerState.UNKNOWN);
    protected final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY);
    protected final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY);
    protected final MediaPlayerFactory mediaPlayerFactory;

    protected T mediaPlayer;
    protected Throwable error;
    protected boolean initialized;

    //region Constructors

    /**
     * Initialize a new abstract video player instance.
     *
     * @param nativeDiscovery The native discovery that needs to be used for VLC.
     */
    protected AbstractVideoPlayer(NativeDiscovery nativeDiscovery) {
        mediaPlayerFactory = createFactory(nativeDiscovery);
    }

    //endregion

    //region Properties

    @Override
    public PlayerState getPlayerState() {
        return playerState.get();
    }

    @Override
    public ReadOnlyObjectProperty<PlayerState> playerStateProperty() {
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
    public ReadOnlyLongProperty timeProperty() {
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
    public ReadOnlyLongProperty durationProperty() {
        return duration;
    }

    protected void setDuration(long duration) {
        this.duration.set(duration);
    }

    //endregion

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

    //endregion

    //region VideoPlayer

    @Override
    public void subtitleFile(File file) {
        mediaPlayer.subpictures().setSubTitleFile(file);
    }

    @Override
    public void subtitleDelay(long delay) {
        log.trace("Updated subtitle delay to {} milliseconds", delay);
        mediaPlayer.subpictures().setDelay(delay * 1000);
    }

    //endregion

    //region Functions

    /**
     * Create a new {@link MediaPlayerFactory} instance.
     *
     * @param nativeDiscovery The native discovery to use for the factory.
     * @return Returns the new media player factory instance.
     */
    protected abstract MediaPlayerFactory createFactory(NativeDiscovery nativeDiscovery);

    protected void initialize() {
        initializeEvents();
    }

    protected void reset() {
        error = null;

        setTime(0);
        setDuration(0);
    }

    protected void setError(Throwable throwable) {
        this.error = throwable;
        setPlayerState(PlayerState.ERROR);
    }

    protected void checkInitialized() {
        if (!initialized)
            throw new VideoPlayerNotInitializedException(this);
    }

    private void initializeEvents() {
        mediaPlayer.events().addMediaPlayerEventListener(new MediaPlayerEventAdapter() {
            @Override
            public void playing(MediaPlayer mediaPlayer) {
                setPlayerState(PlayerState.PLAYING);
            }

            @Override
            public void paused(MediaPlayer mediaPlayer) {
                setPlayerState(PlayerState.PAUSED);
            }

            @Override
            public void stopped(MediaPlayer mediaPlayer) {
                setPlayerState(PlayerState.STOPPED);
            }

            @Override
            public void finished(MediaPlayer mediaPlayer) {
                setPlayerState(PlayerState.FINISHED);
            }

            @Override
            public void buffering(MediaPlayer mediaPlayer, float newCache) {
                log.trace("VLC buffer is now {}%", newCache);
                if (newCache < 100) {
                    setPlayerState(PlayerState.BUFFERING);
                } else {
                    setPlayerState(PlayerState.PLAYING);
                }
            }

            @Override
            public void error(MediaPlayer mediaPlayer) {
                setError(new VideoPlayerException("VLC media player went into error state"));
            }

            @Override
            public void timeChanged(MediaPlayer mediaPlayer, long newTime) {
                setTime(newTime);
            }

            @Override
            public void lengthChanged(MediaPlayer mediaPlayer, long newLength) {
                setDuration(newLength);
            }
        });
    }

    protected void invokeOnVlc(Runnable runnable) {
        mediaPlayer.submit(() -> {
            try {
                runnable.run();
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
                setError(new VideoPlayerException(ex.getMessage(), ex));
            }
        });
    }

    //endregion
}

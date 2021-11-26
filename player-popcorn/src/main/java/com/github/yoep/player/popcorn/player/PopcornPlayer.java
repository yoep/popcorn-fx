package com.github.yoep.player.popcorn.player;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.listeners.VideoListener;
import com.github.yoep.video.adapter.state.VideoState;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import java.util.Collection;
import java.util.Optional;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

@Slf4j
@Component
@RequiredArgsConstructor
public class PopcornPlayer implements Player {
    public static final String PLAYER_ID = "internalPlayer";
    public static final String PLAYER_NAME = "Popcorn Time";

    static final Resource GRAPHIC_RESOURCE = new ClassPathResource("/internal-popcorn-icon.png");

    private final Collection<PlayerListener> listeners = new ConcurrentLinkedQueue<>();
    private final VideoListener videoListener = createVideoListener();

    private PlayerState playerState;
    private PlaybackListener playbackListener;
    private Long time;
    private Long duration;

    //region EmbeddablePlayer

    @Override
    public String getId() {
        return PLAYER_ID;
    }

    @Override
    public String getName() {
        return PLAYER_NAME;
    }

    @Override
    public Optional<Resource> getGraphicResource() {
        return Optional.of(GRAPHIC_RESOURCE);
    }

    @Override
    public PlayerState getState() {
        return null;
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return true;
    }

    @Override
    public void dispose() {
        log.debug("Disposing the popcorn player");
    }

    @Override
    public void addListener(PlayerListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(PlayerListener listener) {
        listeners.remove(listener);
    }

    @Override
    public void play(PlayRequest request) {
        Assert.notNull(request, "request cannot be null");
        invokeListeners(e -> e.onPlay(request));
    }

    @Override
    public void resume() {
        invokeListeners(PlaybackListener::onResume);
    }

    @Override
    public void pause() {
        invokeListeners(PlaybackListener::onPause);
    }

    @Override
    public void stop() {
        log.trace("Stopping video playback");
        invokeListeners(PlaybackListener::onStop);
    }

    @Override
    public void seek(long time) {
        log.trace("Updating video playback time to {}", time);
        invokeListeners(e -> e.onSeek(time));
    }

    @Override
    public void volume(int volume) {
        log.trace("Updating video playback volume to {}", volume);
        invokeListeners(e -> e.onVolume(volume));
    }

    //endregion

    //region Properties

    public void setPlaybackListener(PlaybackListener playbackListener) {
        this.playbackListener = playbackListener;
    }

    //endregion

    //region Methods

    /**
     * Update the active video player of the player.
     *
     * @param oldPlayer The old video playback player.
     * @param newPlayer The new video playback player.
     */
    public void updateActiveVideoPlayer(@Nullable VideoPlayer oldPlayer, @Nullable VideoPlayer newPlayer) {
        Optional.ofNullable(oldPlayer)
                .ifPresent(e -> e.removeListener(videoListener));
        Optional.ofNullable(newPlayer)
                .ifPresent(e -> e.addListener(videoListener));
    }

    //endregion

    //region Functions

    private void invokeListeners(Consumer<PlaybackListener> action) {
        if (playbackListener == null) {
            return;
        }

        try {
            action.accept(playbackListener);
        } catch (Exception ex) {
            log.error("Failed to invoke playback listener, {}", ex.getMessage(), ex);
        }
    }

    private void setPlayerState(PlayerState playerState) {
        this.playerState = playerState;
        listeners.forEach(e -> e.onStateChanged(playerState));
    }

    private void setTime(Long time) {
        this.time = time;
        listeners.forEach(e -> e.onTimeChanged(time));
    }

    private void setDuration(Long duration) {
        this.duration = duration;
        listeners.forEach(e -> e.onDurationChanged(duration));
    }

    private void onVideoStateChanged(VideoState newState) {
        switch (newState) {
            case BUFFERING:
                setPlayerState(PlayerState.BUFFERING);
                break;
            case PLAYING:
                setPlayerState(PlayerState.PLAYING);
                break;
            case PAUSED:
                setPlayerState(PlayerState.PAUSED);
                break;
            case STOPPED:
            case FINISHED:
                setPlayerState(PlayerState.STOPPED);
                break;
            case ERROR:
                setPlayerState(PlayerState.ERROR);
                break;
            case UNKNOWN:
            default:
                setPlayerState(PlayerState.UNKNOWN);
                break;
        }
    }

    private VideoListener createVideoListener() {
        return new VideoListener() {
            @Override
            public void onDurationChanged(long newDuration) {
                setDuration(newDuration);
            }

            @Override
            public void onTimeChanged(long newTime) {
                setTime(newTime);
            }

            @Override
            public void onStateChanged(VideoState newState) {
                onVideoStateChanged(newState);
            }
        };
    }

    //endregion
}

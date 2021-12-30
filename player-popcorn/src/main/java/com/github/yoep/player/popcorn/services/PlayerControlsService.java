package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.AbstractPlayerListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerControlsService extends AbstractListenerService<PlayerControlsListener> {
    private final PopcornPlayer player;
    private final ScreenService screenService;
    private final VideoService videoService;

    private final PlaybackListener playbackListener = createPlaybackListener();
    private final PlayerListener playerListener = createPlayerListener();

    //region Methods

    public void toggleFullscreen() {
        screenService.toggleFullscreen();
    }

    public void togglePlayerPlaybackState() {
        if (player.getState() == PlayerState.PAUSED) {
            player.resume();
        } else {
            player.pause();
        }
    }

    public void onSeekChanging(Boolean isSeeking) {
        if (isSeeking) {
            player.pause();
        } else {
            player.resume();
        }
    }

    public void seek(long time) {
        player.seek(time);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        screenService.fullscreenProperty().addListener((observableValue, oldValue, newValue) -> invokeListeners(e -> e.onFullscreenStateChanged(newValue)));
        videoService.addListener(playbackListener);
        player.addListener(playerListener);
    }

    //endregion

    //region Functions

    private void onPlayRequest(PlayRequest request) {
        invokeListeners(e -> e.onSubtitleStateChanged(request.isSubtitlesEnabled()));
    }

    private void onPlayerStateChanged(PlayerState state) {
        invokeListeners(e -> e.onPlayerStateChanged(state));
    }

    private void onPlayerTimeChanged(long time) {
        invokeListeners(e -> e.onPlayerTimeChanged(time));
    }

    private void onPlayerDurationChanged(long duration) {
        invokeListeners(e -> e.onPlayerDurationChanged(duration));
    }

    private PlayerListener createPlayerListener() {
        return new AbstractPlayerListener() {
            @Override
            public void onStateChanged(PlayerState newState) {
                onPlayerStateChanged(newState);
            }

            @Override
            public void onTimeChanged(long newTime) {
                onPlayerTimeChanged(newTime);
            }

            @Override
            public void onDurationChanged(long newDuration) {
                onPlayerDurationChanged(newDuration);
            }
        };
    }

    private PlaybackListener createPlaybackListener() {
        return new AbstractPlaybackListener() {
            @Override
            public void onPlay(PlayRequest request) {
                onPlayRequest(request);
            }
        };
    }

    //endregion
}

package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.player.popcorn.controllers.components.PlayerControlsComponent;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.List;

/**
 * The playback event service is for handling the events triggered by the playback.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PlaybackEventService {
    private final PopcornPlayer popcornPlayer;
    private final PlayerControlsComponent playerControls;
    private final List<PlaybackListener> listeners;

    private final PlayerListener playerListener = createPlayerListener();
    private final PlaybackListener playbackListener = createPlaybackListener();

    //region PostConstruct

    @PostConstruct
    void init() {
        initializePlayerListener();
        initializePlaybackListener();
    }

    //endregion

    //region Functions

    private void initializePlayerListener() {
        popcornPlayer.addListener(playerListener);
    }

    private void initializePlaybackListener() {
        popcornPlayer.setPlaybackListener(playbackListener);
    }

    private void onPlayerDurationChanged(Long duration) {
        playerControls.updateDuration(duration);
    }

    private void onPlayerTimeChanged(Long time) {
        playerControls.updateTime(time);
    }

    private void onPlayerStateChanged(PlayerState newState) {
        playerControls.updatePlaybackState(newState != PlayerState.PAUSED);
    }

    private PlayerListener createPlayerListener() {
        return new PlayerListener() {
            @Override
            public void onDurationChanged(long newDuration) {
                onPlayerDurationChanged(newDuration);
            }

            @Override
            public void onTimeChanged(long newTime) {
                onPlayerTimeChanged(newTime);
            }

            @Override
            public void onStateChanged(PlayerState newState) {
                onPlayerStateChanged(newState);
            }
        };
    }

    private PlaybackListener createPlaybackListener() {
        return new PlaybackListener() {
            @Override
            public void onPlay(PlayRequest request) {
                listeners.forEach(e -> {
                    try {
                        e.onPlay(request);
                    } catch (Exception ex) {
                        log.error("Failed to invoke onPlay, {}", ex.getMessage(), ex);
                    }
                });
            }

            @Override
            public void onResume() {
                listeners.forEach(e -> {
                    try {
                        e.onResume();
                    } catch (Exception ex) {
                        log.error("Failed to invoke onResume, {}", ex.getMessage(), ex);
                    }
                });
            }

            @Override
            public void onPause() {
                listeners.forEach(e -> {
                    try {
                        e.onPause();
                    } catch (Exception ex) {
                        log.error("Failed to invoke onPause, {}", ex.getMessage(), ex);
                    }
                });
            }

            @Override
            public void onSeek(long time) {
                listeners.forEach(e -> {
                    try {
                        e.onSeek(time);
                    } catch (Exception ex) {
                        log.error("Failed to invoke onPause, {}", ex.getMessage(), ex);
                    }
                });
            }

            @Override
            public void onVolume(int volume) {
                listeners.forEach(e -> {
                    try {
                        e.onVolume(volume);
                    } catch (Exception ex) {
                        log.error("Failed to invoke onVolume, {}", ex.getMessage(), ex);
                    }
                });
            }

            @Override
            public void onStop() {
                listeners.forEach(e -> {
                    try {
                        e.onStop();
                    } catch (Exception ex) {
                        log.error("Failed to invoke onStop, {}", ex.getMessage(), ex);
                    }
                });
            }
        };
    }

    //endregion
}

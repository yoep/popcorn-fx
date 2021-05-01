package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.player.popcorn.controllers.components.PlayerControlsComponent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

/**
 * The playback event service is for handling the events triggered by the playback.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PlaybackEventService {
    private final RegisterService registerService;
    private final PlayerControlsComponent playerControls;

    private final PlayerListener listener = createListener();

    //region PostConstruct

    @PostConstruct
    void init() {
        initializePlayerListener();
    }

    //endregion

    //region Functions

    private void initializePlayerListener() {
        registerService.getPlayer().addListener(listener);
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

    private PlayerListener createListener() {
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

    //endregion
}

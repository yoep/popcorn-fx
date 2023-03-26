package com.github.yoep.popcorn.backend.controls;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.player.PlayerEventService;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedDeque;

@Slf4j
public class PlaybackControlsService {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final PlayerManagerService playerManagerService;
    private final PlayerEventService playerEventService;

    private final PlaybackControlCallback callback = createCallbackListener();
    private final Queue<PlaybackControlCallback> listeners = new ConcurrentLinkedDeque<>();

    private long lastKnownTime;

    public PlaybackControlsService(FxLib fxLib, PopcornFx instance, PlayerManagerService playerManagerService, PlayerEventService playerEventService) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.playerManagerService = playerManagerService;
        this.playerEventService = playerEventService;
        init();
    }

    public void register(PlaybackControlCallback callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    private void init() {
        playerEventService.addListener(new PlayerListener() {
            @Override
            public void onDurationChanged(long newDuration) {

            }

            @Override
            public void onTimeChanged(long newTime) {
                lastKnownTime = newTime;
            }

            @Override
            public void onStateChanged(PlayerState newState) {

            }

            @Override
            public void onVolumeChanged(int volume) {

            }
        });
        fxLib.register_playback_controls(instance, callback);
    }

    private void onSeekMediaTime(long offset) {
        playerManagerService.getActivePlayer()
                .ifPresent(e -> e.seek(lastKnownTime + offset));
    }

    private void onCallback(PlaybackControlEvent event) {
        switch (event) {
            case TogglePlaybackState -> playerManagerService.getActivePlayer().ifPresent(e -> {
                if (e.getState() == PlayerState.PLAYING) {
                    e.pause();
                } else {
                    e.resume();
                }
            });
            case Forward -> onSeekMediaTime(10000);
            case Rewind -> onSeekMediaTime(-10000);
        }
    }

    private PlaybackControlCallback createCallbackListener() {
        return event -> {
            log.debug("Received playback control event callback {}", event);
            onCallback(event);

            new Thread(() -> {
                for (var listener : listeners) {
                    try {
                        listener.callback(event);
                    } catch (Exception ex) {
                        log.error("Failed to invoke favorite callback, {}", ex.getMessage(), ex);
                    }
                }
            }, "PlaybackControlCallbackHandler").start();
        };
    }
}

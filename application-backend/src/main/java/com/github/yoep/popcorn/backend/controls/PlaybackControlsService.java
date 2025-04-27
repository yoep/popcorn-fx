package com.github.yoep.popcorn.backend.controls;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ControlEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.player.PlayerEventService;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedDeque;

@Slf4j
public class PlaybackControlsService implements FxCallback<ControlEvent> {
    private final FxChannel fxChannel;
    private final PlayerManagerService playerManagerService;
    private final PlayerEventService playerEventService;

    private final Queue<FxCallback<ControlEvent>> listeners = new ConcurrentLinkedDeque<>();

    long lastKnownTime;

    public PlaybackControlsService(FxChannel fxChannel, PlayerManagerService playerManagerService, PlayerEventService playerEventService) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
        this.playerManagerService = playerManagerService;
        this.playerEventService = playerEventService;
        init();
    }

    public void register(FxCallback<ControlEvent> callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    @Override
    public void callback(ControlEvent event) {
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
            public void onStateChanged(Player.State newState) {

            }

            @Override
            public void onVolumeChanged(int volume) {

            }
        });
        fxChannel.subscribe(
                FxChannel.typeFrom(ControlEvent.class),
                ControlEvent.parser(),
                this
        );
    }

    private void onSeekMediaTime(long offset) {
        playerManagerService.getActivePlayer().whenComplete((player, throwable) -> {
            if (throwable == null) {
                player.ifPresent(e -> e.seek(lastKnownTime + offset));
            } else {
                log.error("Failed to retrieve active player", throwable);
            }
        });
    }

    private void onCallback(ControlEvent event) {
        switch (event.getEvent()) {
            case TOGGLE_PLAYBACK_STATE -> playerManagerService.getActivePlayer().whenComplete((player, throwable) -> {
                if (throwable == null) {
                    player.ifPresent(e -> {
                        if (e.getState() == Player.State.PLAYING) {
                            e.pause();
                        } else {
                            e.resume();
                        }
                    });
                } else {
                    log.error("Failed to retrieve active player", throwable);
                }
            });
            case FORWARD -> onSeekMediaTime(10000);
            case REWIND -> onSeekMediaTime(-10000);
            default -> {
            }
        }
    }
}

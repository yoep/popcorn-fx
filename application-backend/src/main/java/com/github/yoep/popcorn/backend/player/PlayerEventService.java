package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStateEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.PlayerManagerEvent;
import lombok.extern.slf4j.Slf4j;

import java.util.Collection;
import java.util.Objects;
import java.util.concurrent.ConcurrentLinkedQueue;

/**
 * The player event service is responsible for listening on the player events and translate them to application events when needed.
 * This service will also act as a central non-player aware listener for other services to use when interested to listen on player events
 * and only want to register ones instead of each time a different player is used.
 */
@Slf4j
public class PlayerEventService {
    private final PlayerManagerService playerManager;
    private final EventPublisher eventPublisher;

    private final PlayerManagerListener playerManagerListener = createManagerListener();
    private final Collection<PlayerListener> listeners = new ConcurrentLinkedQueue<>();

    public PlayerEventService(PlayerManagerService playerManager, EventPublisher eventPublisher) {
        Objects.requireNonNull(playerManager, "playerManager cannot be null");
        Objects.requireNonNull(eventPublisher, "eventPublisher cannot be null");
        this.playerManager = playerManager;
        this.eventPublisher = eventPublisher;
        init();
    }

    //region Methods

    public void addListener(PlayerListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    public void removeListener(PlayerListener listener) {
        listeners.remove(listener);
    }

    //endregion

    //region PostConstruct

    private void init() {
        playerManager.addListener(playerManagerListener);
        eventPublisher.register(ClosePlayerEvent.class, event -> {
            playerManager.getActivePlayer().whenComplete((player, throwable) -> {
                if (throwable == null) {
                    player.ifPresent(com.github.yoep.popcorn.backend.adapters.player.Player::stop);
                } else {
                    log.error("Failed to retrieve active player", throwable);
                }
            });
            return event;
        });
    }

    //endregion

    //region Functions

    private void onPlayerStateChanged(Player.State newState) {
        listeners.forEach(e -> e.onStateChanged(newState));
        eventPublisher.publish(new PlayerStateEvent(this, newState));
    }

    private void onPlayerDurationChanged(long duration) {
        listeners.forEach(e -> e.onDurationChanged(duration));
    }

    private void onPlayerTimeChanged(long time) {
        listeners.forEach(e -> e.onTimeChanged(time));
    }

    private PlayerManagerListener createManagerListener() {
        return new PlayerManagerListener() {
            @Override
            public void activePlayerChanged(PlayerManagerEvent.ActivePlayerChanged playerChange) {
                // no-op
            }

            @Override
            public void playersChanged() {
                // no-op
            }

            @Override
            public void onPlayerPlaybackChanged(Player.PlayRequest request) {
                // no-op
            }

            @Override
            public void onPlayerTimeChanged(Long newTime) {
                PlayerEventService.this.onPlayerTimeChanged(newTime);
            }

            @Override
            public void onPlayerDurationChanged(Long newDuration) {
                PlayerEventService.this.onPlayerDurationChanged(newDuration);
            }

            @Override
            public void onPlayerStateChanged(Player.State newState) {
                PlayerEventService.this.onPlayerStateChanged(newState);
            }
        };
    }

    //endregion
}

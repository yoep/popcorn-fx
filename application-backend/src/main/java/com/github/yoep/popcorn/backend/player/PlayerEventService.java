package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStateEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.util.Collection;
import java.util.Optional;
import java.util.concurrent.ConcurrentLinkedQueue;

/**
 * The player event service is responsible for listening on the player events and translate them to application events when needed.
 * This service will also act as a central non-player aware listener for other services to use when interested to listen on player events
 * and only want to register ones instead of each time a different player is used.
 */
@Slf4j
@RequiredArgsConstructor
public class PlayerEventService {
    private final PlayerManagerService playerService;
    private final EventPublisher eventPublisher;

    private final PlayerListener playerListener = createListener();
    private final Collection<PlayerListener> listeners = new ConcurrentLinkedQueue<>();

    //region Methods

    public void addListener(PlayerListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    public void removeListener(PlayerListener listener) {
        listeners.remove(listener);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        playerService.activePlayerProperty().addListener((observable, oldValue, newValue) -> onPlayerChanged(oldValue, newValue));
        eventPublisher.register(ClosePlayerEvent.class, event -> {
            playerService.getActivePlayer()
                    .ifPresent(Player::stop);
            return event;
        });
    }

    //endregion

    //region Functions

    private void onPlayerChanged(Player oldValue, Player newValue) {
        log.debug("Active player has been changed to {}, updating the player listener", newValue.getId());
        // check if we need to unregister the listener from the old player
        Optional.ofNullable(oldValue)
                .ifPresent(e -> e.removeListener(playerListener));
        Optional.ofNullable(newValue)
                .ifPresent(e -> e.addListener(playerListener));
    }

    private void onPlayerStateChanged(PlayerState newState) {
        listeners.forEach(e -> e.onStateChanged(newState));
        eventPublisher.publish(new PlayerStateEvent(this, newState));
    }

    private void onPlayerDurationChanged(long duration) {
        listeners.forEach(e -> e.onDurationChanged(duration));
    }

    private void onPlayerTimeChanged(long time) {
        listeners.forEach(e -> e.onTimeChanged(time));
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

            @Override
            public void onVolumeChanged(int volume) {
                // no-op
            }
        };
    }

    //endregion
}
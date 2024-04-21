package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PreDestroy;
import java.util.*;

/**
 * Implementation of the {@link PlayerManagerService} which serves the individual players with a central point of management.
 * This service manages each available {@link Player} of the application.
 */
@Slf4j
public class PlayerManagerServiceImpl extends AbstractListenerService<PlayerManagerListener> implements PlayerManagerService, PlayerManagerCallback {
    private final List<PlayerWrapper> playerWrappers = new ArrayList<>();
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final EventPublisher eventPublisher;

    public PlayerManagerServiceImpl(FxLib fxLib, PopcornFx instance, EventPublisher eventPublisher) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.eventPublisher = eventPublisher;
        init();
    }

    //region Properties

    @Override
    public Optional<Player> getById(String id) {
        Objects.requireNonNull(id, "id cannot be null");
        try (var player = fxLib.player_by_id(instance, id)) {
            return Optional.ofNullable(player)
                    .map(this::enhance);
        }
    }

    @Override
    public Collection<Player> getPlayers() {
        try (var players = fxLib.players(instance)) {
            return players.getPlayers().stream()
                    .map(this::enhance)
                    .toList();
        }
    }

    @Override
    public Optional<Player> getActivePlayer() {
        var player = fxLib.active_player(instance);
        return Optional.ofNullable(player)
                .map(this::enhance);
    }

    @Override
    public void setActivePlayer(Player activePlayer) {
        Objects.requireNonNull(activePlayer, "activePlayer is required");
        log.trace("Activating player {} for playbacks", activePlayer);
        fxLib.set_active_player(instance, activePlayer.getId());
    }

    //endregion

    //region Methods

    @Override
    public void register(Player player) {
        Objects.requireNonNull(player, "player cannot be null");
        log.trace("Registering new player {}", player);
        try (var wrapper = new PlayerWrapperRegistration.ByValue(player)) {
            fxLib.register_player(instance, wrapper);
            wrapper.setPlayerC(fxLib.player_pointer_by_id(instance, player.getId()));
            wrapper.setListener(new PlayerListener() {
                @Override
                public void onDurationChanged(long newDuration) {
                    try (var event = PlayerEventC.ByValue.durationChanged(newDuration)) {
                        fxLib.invoke_player_event(wrapper.playerC, event);
                    }
                }

                @Override
                public void onTimeChanged(long newTime) {
                    try (var event = PlayerEventC.ByValue.timeChanged(newTime)) {
                        fxLib.invoke_player_event(wrapper.playerC, event);
                    }
                }

                @Override
                public void onStateChanged(PlayerState newState) {
                    try (var event = PlayerEventC.ByValue.stateChanged(newState)) {
                        fxLib.invoke_player_event(wrapper.playerC, event);
                    }
                }

                @Override
                public void onVolumeChanged(int volume) {

                }
            });
            playerWrappers.add(wrapper);
        }
    }

    @Override
    public void unregister(Player player) {
        log.trace("Removing player \"{}\"", player);
        fxLib.remove_player(instance, player.getId());
        playerWrappers.stream()
                .filter(e -> Objects.equals(e.getId(), player.getId()))
                .findFirst()
                .map(playerWrappers::remove);
    }

    @Override
    public void callback(PlayerManagerEvent.ByValue event) {
        log.debug("Received player manager event {}", event);
        try (event) {
            invokeListeners(listener -> {
                switch (event.getTag()) {
                    case ACTIVE_PLAYER_CHANGED -> {
                        var change = event.getUnion().getPlayerChanged_body().playerChangedEvent;
                        listener.activePlayerChanged(new PlayerChanged(change.getOldPlayerId().orElse(null), change.getNewPlayerId(),
                                change.getNewPlayerName()));
                    }
                    case PLAYERS_CHANGED -> listener.playersChanged();
                    case PLAYER_PLAYBACK_CHANGED -> listener.onPlayerPlaybackChanged(event.getUnion().getPlayerPlaybackChanged_body().getRequest());
                    case PLAYER_TIME_CHANGED -> listener.onPlayerTimeChanged(event.getUnion().getPlayerTimeChanged_body().getTime());
                    case PLAYER_DURATION_CHANGED -> listener.onPlayerDurationChanged(event.getUnion().getPlayerDurationChanged_body().getDuration());
                    case PLAYER_STATE_CHANGED -> listener.onPlayerStateChanged(event.getUnion().getPlayerStateChanged_body().getState());
                }
            });
        }
    }

    //endregion

    //region OnDestroy

    @PreDestroy
    void onDestroy() {
        log.debug("Disposing all player resources");
        playerWrappers.forEach(Player::dispose);
    }

    //endregion

    void init() {
        registerCallbackHandler();

        registerEventListeners();
    }

    private void registerCallbackHandler() {
        try {
            log.debug("Registering player manager C callback");
            fxLib.register_player_callback(instance, this);
        } catch (Exception ex) {
            log.error("Failed to register player manager callback handler, {}", ex.getMessage(), ex);
        }
    }

    private void registerEventListeners() {
        eventPublisher.register(ClosePlayerEvent.class, closePlayerEvent -> {
            if (closePlayerEvent.getReason() == ClosePlayerEvent.Reason.USER) {
                getActivePlayer().ifPresentOrElse(
                        Player::stop,
                        () -> log.warn("Unable to stop player, no active player present")
                );
            }

            return closePlayerEvent;
        }, EventPublisher.HIGHEST_ORDER);
    }

    private Player enhance(Player player) {
        if (player instanceof PlayerWrapper wrapper) {
            if (wrapper.getPlayerC() == null) {
                wrapper.setPlayerC(fxLib.player_pointer_by_id(instance, player.getId()));
            }
            return playerWrappers.stream()
                    .filter(e -> Objects.equals(e.getId(), wrapper.getId()))
                    .findFirst()
                    .map(PlayerWrapper::getPlayer)
                    .orElse(wrapper);
        }
        return player;
    }
}

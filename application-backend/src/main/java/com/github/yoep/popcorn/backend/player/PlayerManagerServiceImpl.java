package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

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
    private final ApplicationConfig applicationConfig;
    private final ScreenService screenService;

    public PlayerManagerServiceImpl(FxLib fxLib, PopcornFx instance, ApplicationConfig applicationConfig, ScreenService screenService) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.applicationConfig = applicationConfig;
        this.screenService = screenService;
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
        Assert.notNull(player, "player cannot be null");
        log.trace("Registering new player {}", player);
        try (var wrapper = new PlayerWrapperRegistration(player)) {
            var playerC = fxLib.register_player(instance, wrapper);
            wrapper.setPlayerC(playerC);
            wrapper.setListener(new PlayerListener() {
                @Override
                public void onDurationChanged(long newDuration) {
                    var event = PlayerEventC.ByValue.durationChanged(newDuration);
                    fxLib.invoke_player_event(wrapper.playerC, event);
                }

                @Override
                public void onTimeChanged(long newTime) {
                    var event = PlayerEventC.ByValue.timeChanged(newTime);
                    fxLib.invoke_player_event(wrapper.playerC, event);
                }

                @Override
                public void onStateChanged(PlayerState newState) {

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
    public void callback(PlayerManagerEvent event) {
        log.debug("Received player manager event {}", event);
        invokeListeners(listener -> {
            switch (event.getTag()) {
                case ActivePlayerChanged -> {
                    var change = event.getUnion().getPlayerChanged_body().playerChangedEvent;
                    listener.activePlayerChanged(new PlayerChanged(change.getOldPlayerId().orElse(null), change.getNewPlayerId(), change.getNewPlayerName()));
                }
                case PlayersChanged -> listener.playersChanged();
            }
        });
    }

    //endregion

    //region OnDestroy

    @PreDestroy
    void onDestroy() {
        log.debug("Disposing all player resources");
        playerWrappers.forEach(Player::dispose);
    }

    //endregion

    private void init() {
        log.debug("Registering player manager C callback");
        fxLib.register_player_callback(instance, this);
    }

    private Player enhance(Player player) {
        if (player instanceof PlayerWrapper wrapper) {
            return playerWrappers.stream()
                    .filter(e -> Objects.equals(e.getId(), wrapper.getId()))
                    .findFirst()
                    .map(PlayerWrapper::getPlayer)
                    .orElse(wrapper);
        }
        return player;
    }

    private void fullscreenVideo() {
        var settings = applicationConfig.getSettings();
        var playbackSettings = settings.getPlaybackSettings();

        if (playbackSettings.isFullscreen()) {
            screenService.fullscreen(true);
        }
    }
}

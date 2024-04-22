package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.AbstractPlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import com.github.yoep.popcorn.backend.player.PlayerChanged;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;

@Slf4j
public class PlayerInfoService extends AbstractInfoService {
    private final PlayerManagerService playerManagerService;

    public PlayerInfoService(PlayerManagerService playerManagerService) {
        Objects.requireNonNull(playerManagerService, "playerManagerService cannot be null");
        this.playerManagerService = playerManagerService;
        init();
    }

    private void init() {
        playerManagerService.addListener(new PlayerManagerListener() {
            @Override
            public void activePlayerChanged(PlayerChanged playerChange) {
                // no-op
            }

            @Override
            public void playersChanged() {
                onPlayersChanged(playerManagerService.getPlayers().stream().toList());
            }

            @Override
            public void onPlayerPlaybackChanged(PlayRequest request) {
                // no-op
            }

            @Override
            public void onPlayerTimeChanged(Long newTime) {
                // no-op
            }

            @Override
            public void onPlayerDurationChanged(Long newDuration) {
                // no-op
            }

            @Override
            public void onPlayerStateChanged(PlayerState newState) {
                // no-op
            }
        });
    }

    private void onPlayersChanged(List<Player> players) {
        updateComponents(players.stream()
                .map(this::createComponentDetails)
                .collect(Collectors.toList()));
    }

    private SimpleComponentDetails createComponentDetails(Player player) {
        var componentDetails = SimpleComponentDetails.builder()
                .name(player.getName())
                .description(player.getDescription())
                .state(mapToComponentState(player.getState()))
                .build();

        player.addListener(new AbstractPlayerListener() {
            @Override
            public void onStateChanged(PlayerState newState) {
                componentDetails.setState(mapToComponentState(newState));
            }
        });

        return componentDetails;
    }

    private static ComponentState mapToComponentState(PlayerState state) {
        return Optional.ofNullable(state)
                .map(e -> switch (e) {
                    case ERROR -> ComponentState.ERROR;
                    case UNKNOWN -> ComponentState.UNKNOWN;
                    default -> ComponentState.READY;
                })
                .orElse(ComponentState.UNKNOWN);
    }
}

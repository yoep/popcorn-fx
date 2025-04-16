package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.PlayerManagerEvent;
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
            public void activePlayerChanged(PlayerManagerEvent.ActivePlayerChanged playerChange) {
                // no-op
            }

            @Override
            public void playersChanged() {
                playerManagerService.getPlayers().whenComplete((players, throwable) -> {
                    if (throwable == null) {
                        onPlayersChanged(players.stream().toList());
                    } else {
                        log.error("Failed to retrieve players", throwable);
                    }
                });
            }

            @Override
            public void onPlayerPlaybackChanged(Player.PlayRequest request) {
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
            public void onPlayerStateChanged(Player.State newState) {
                // no-op
            }
        });
    }

    private void onPlayersChanged(List<com.github.yoep.popcorn.backend.adapters.player.Player> players) {
        updateComponents(players.stream()
                .map(this::createComponentDetails)
                .collect(Collectors.toList()));
    }

    private SimpleComponentDetails createComponentDetails(com.github.yoep.popcorn.backend.adapters.player.Player player) {
        var componentDetails = SimpleComponentDetails.builder()
                .name(player.getName())
                .description(player.getDescription())
                .state(mapToComponentState(player.getState()))
                .build();

//        player.addListener(new AbstractPlayerListener() {
//            @Override
//            public void onStateChanged(PlayerState newState) {
//                componentDetails.setState(mapToComponentState(newState));
//            }
//        });

        return componentDetails;
    }

    private static ComponentState mapToComponentState(Player.State state) {
        return Optional.ofNullable(state)
                .map(e -> switch (e) {
                    case ERROR -> ComponentState.ERROR;
                    case UNKNOWN -> ComponentState.UNKNOWN;
                    default -> ComponentState.READY;
                })
                .orElse(ComponentState.UNKNOWN);
    }
}

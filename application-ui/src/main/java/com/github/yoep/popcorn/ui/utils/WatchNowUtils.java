package com.github.yoep.popcorn.ui.utils;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.PlayerManagerEvent;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.ui.view.controls.DropDownButton;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.application.Platform;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.util.Collection;
import java.util.Objects;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class WatchNowUtils {
    public static void syncPlayerManagerAndWatchNowButton(PlayerManagerService playerManagerService,
                                                          PlayerDropDownButton watchNowButton) {
        Objects.requireNonNull(playerManagerService, "playerManagerService cannot be null");
        Objects.requireNonNull(watchNowButton, "watchNowButton cannot be null");

        // listen for changes in the players
        playerManagerService.addListener(new PlayerManagerListener() {
            @Override
            public void activePlayerChanged(PlayerManagerEvent.ActivePlayerChanged playerChange) {
                watchNowButton.getDropDownItems().stream()
                        .filter(e -> Objects.equals(e.getId(), playerChange.getNewPlayerId()))
                        .findFirst()
                        .ifPresent(player -> Platform.runLater(() -> watchNowButton.select(player)));
            }

            @Override
            public void playersChanged() {
                updateAvailablePlayers(playerManagerService, watchNowButton);
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

        // create initial list for the current known external players
        updateAvailablePlayers(playerManagerService, watchNowButton);

        // listen on player selection changed
        watchNowButton.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            // verify if the new value is not null
            // if so, update the active player
            if (newValue != null)
                playerManagerService.setActivePlayer(newValue);
        });
    }

    private static void updateAvailablePlayers(PlayerManagerService playerManagerService, DropDownButton<com.github.yoep.popcorn.backend.adapters.player.Player> watchNowButton) {
        playerManagerService.getPlayers()
                .thenCompose(players -> playerManagerService.getActivePlayer()
                        .thenApply(player -> new PlayersInfo(players, player.orElse(null))))
                .thenAccept(info -> Platform.runLater(() -> {
                    watchNowButton.clear();
                    watchNowButton.addDropDownItems(info.players);
                    watchNowButton.select(info.activePlayer);
                }))
                .exceptionally(ex -> {
                    log.error("Failed to retrieve available players", ex);
                    return null;
                });
    }

    private record PlayersInfo(Collection<com.github.yoep.popcorn.backend.adapters.player.Player> players,
                               com.github.yoep.popcorn.backend.adapters.player.Player activePlayer) {
    }
}

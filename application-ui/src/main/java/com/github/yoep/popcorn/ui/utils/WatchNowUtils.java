package com.github.yoep.popcorn.ui.utils;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.player.PlayerChanged;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.ui.view.controls.DropDownButton;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.application.Platform;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class WatchNowUtils {
    public static void syncPlayerManagerAndWatchNowButton(PlayerManagerService playerManagerService,
                                                          PlayerDropDownButton watchNowButton) {
        Objects.requireNonNull(playerManagerService, "playerManagerService cannot be null");
        Objects.requireNonNull(watchNowButton, "watchNowButton cannot be null");

        // listen for changes in the players
        playerManagerService.addListener(new PlayerManagerListener() {
            @Override
            public void activePlayerChanged(PlayerChanged playerChange) {
                playerManagerService.getById(playerChange.newPlayerId())
                        .ifPresent(watchNowButton::select);
            }

            @Override
            public void playersChanged() {
                updateExternalPlayers(playerManagerService, watchNowButton);
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

        // create initial list for the current known external players
        updateExternalPlayers(playerManagerService, watchNowButton);

        // listen on player selection changed
        watchNowButton.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            // verify if the new value is not null
            // if so, update the active player
            if (newValue != null)
                playerManagerService.setActivePlayer(newValue);
        });
    }

    private static void updateExternalPlayers(PlayerManagerService playerManagerService, DropDownButton<Player> watchNowButton) {
        Platform.runLater(() -> {
            watchNowButton.clear();
            watchNowButton.addDropDownItems(playerManagerService.getPlayers());
            watchNowButton.select(playerManagerService.getActivePlayer().orElse(null));
        });
    }
}

package com.github.yoep.popcorn.ui.utils;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.ui.view.controls.DropDownButton;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.beans.InvalidationListener;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class WatchNowUtils {
    public static void syncPlayerManagerAndWatchNowButton(PlatformProvider platformProvider,
                                                          PlayerManagerService playerManagerService,
                                                          PlayerDropDownButton watchNowButton) {
        Objects.requireNonNull(playerManagerService, "playerManagerService cannot be null");
        Objects.requireNonNull(watchNowButton, "watchNowButton cannot be null");

        // listen for changes in the players
        playerManagerService.playersProperty().addListener((InvalidationListener) change ->
                updateExternalPlayers(platformProvider, playerManagerService, watchNowButton));
        playerManagerService.activePlayerProperty().addListener((observable, oldValue, newValue) -> watchNowButton.select(newValue));

        // create initial list for the current known external players
        updateExternalPlayers(platformProvider, playerManagerService, watchNowButton);

        // listen on player selection changed
        watchNowButton.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            // verify if the new value is not null
            // if so, update the active player
            if (newValue != null)
                playerManagerService.setActivePlayer(newValue);
        });
    }

    private static void updateExternalPlayers(PlatformProvider platformProvider, PlayerManagerService playerManagerService, DropDownButton<Player> watchNowButton) {
        platformProvider.runOnRenderer(() -> {
            watchNowButton.clear();
            watchNowButton.addDropDownItems(playerManagerService.getPlayers());
            watchNowButton.select(playerManagerService.getActivePlayer().orElse(null));
        });
    }
}

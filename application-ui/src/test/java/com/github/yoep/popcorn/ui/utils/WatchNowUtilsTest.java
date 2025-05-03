package com.github.yoep.popcorn.ui.utils;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.collections.FXCollections;
import javafx.collections.ObservableMap;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.util.LinkedHashMap;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static java.util.Arrays.asList;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class WatchNowUtilsTest {
    @Mock
    private PlayerManagerService playerManagerService;
    private PlayerDropDownButton watchNowButton;

    private final ObservableMap<String, Player> playerProperty = FXCollections.observableMap(new LinkedHashMap<>());

    @BeforeEach
    void setUp() {
        watchNowButton = new PlayerDropDownButton();
    }

    @Test
    void testSynchronize_whenPlayersAreChanged_shouldUpdatePlayers() throws TimeoutException {
        var player = mock(Player.class);
        var player2 = mock(Player.class);
        var players = asList(player2, player);
        when(playerManagerService.getPlayers()).thenReturn(CompletableFuture.completedFuture(players));
        when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.empty()));
        when(player.getId()).thenReturn("Player001");
        when(player2.getId()).thenReturn("Player002");
        WatchNowUtils.syncPlayerManagerAndWatchNowButton(playerManagerService, watchNowButton);
        WaitForAsyncUtils.waitForFxEvents();

        playerProperty.put("myPlayer", player);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> watchNowButton.getDropDownItems().containsAll(players));
    }

    @Test
    void testSynchronize_whenSelectItemIsChanged_shouldUpdatePlayerManager() {
        var player = mock(Player.class);
        var player2 = mock(Player.class);
        var players = asList(player2, player);
        when(playerManagerService.getPlayers()).thenReturn(CompletableFuture.completedFuture(players));
        when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.empty()));
        when(player.getId()).thenReturn("Player001");
        when(player2.getId()).thenReturn("Player002");
        WatchNowUtils.syncPlayerManagerAndWatchNowButton(playerManagerService, watchNowButton);
        WaitForAsyncUtils.waitForFxEvents();

        watchNowButton.select(player2);

        verify(playerManagerService).setActivePlayer(player2);
    }
}
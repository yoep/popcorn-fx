package com.github.yoep.popcorn.ui.utils;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableMap;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.LinkedHashMap;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class WatchNowUtilsIT {
    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private PlayerManagerService playerManagerService;
    private PlayerDropDownButton watchNowButton;

    private final ObservableMap<String, Player> playerProperty = FXCollections.observableMap(new LinkedHashMap<>());
    private final ObjectProperty<Player> activePlayerProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        when(playerManagerService.playersProperty()).thenReturn(playerProperty);
        when(playerManagerService.activePlayerProperty()).thenReturn(activePlayerProperty);
        doAnswer(invocation -> {
            invocation.getArgument(0, Runnable.class).run();
            return null;
        }).when(platformProvider).runOnRenderer(isA(Runnable.class));

        watchNowButton = new PlayerDropDownButton();
    }

    @Test
    void testSynchronize_whenPlayersAreChanged_shouldUpdatePlayers() {
        var player = mock(Player.class);
        var players = asList(mock(Player.class), player);
        when(playerManagerService.getPlayers()).thenReturn(players);
        WatchNowUtils.syncPlayerManagerAndWatchNowButton(playerManagerService, watchNowButton);

        playerProperty.put("myPlayer", player);

        assertTrue(watchNowButton.getDropDownItems().containsAll(players), "Expected the players to have been present");
    }

    @Test
    void testSynchronize_whenSelectItemIsChanged_shouldUpdatePlayerManager() {
        var player = mock(Player.class);
        var player2 = mock(Player.class);
        var players = asList(player2, player);
        when(playerManagerService.getPlayers()).thenReturn(players);
        WatchNowUtils.syncPlayerManagerAndWatchNowButton(playerManagerService, watchNowButton);

        watchNowButton.select(player2);

        verify(playerManagerService).setActivePlayer(player2);
    }
}
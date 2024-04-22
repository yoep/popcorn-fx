package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerInfoServiceTest {
    @Mock
    private PlayerManagerService playerManagerService;
    private final AtomicReference<PlayerListener> listenerHolder = new AtomicReference<>();
    private final AtomicReference<PlayerManagerListener> playerListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            playerListenerHolder.set(invocation.getArgument(0, PlayerManagerListener.class));
            return null;
        }).when(playerManagerService).addListener(isA(PlayerManagerListener.class));
    }

    @Test
    void testListener_whenPlayerAreChanged_shouldUpdateComponentDetailsList() {
        var name = "player-name";
        var description = "player-description";
        var state = PlayerState.READY;
        var player = mock(Player.class);
        var expectedResult = SimpleComponentDetails.builder()
                .name(name)
                .description(description)
                .state(ComponentState.READY)
                .build();
        when(playerManagerService.getPlayers()).thenReturn(Collections.singletonList(player));
        when(player.getName()).thenReturn(name);
        when(player.getDescription()).thenReturn(description);
        when(player.getState()).thenReturn(state);
        var service = new PlayerInfoService(playerManagerService);

        playerListenerHolder.get().playersChanged();
        var result = service.getComponentDetails();

        assertEquals(1, result.size());
        assertEquals(expectedResult, result.get(0));
    }

    @Test
    void testListener_whenPlayerStateChanges_shouldChangeDetailState() {
        var name = "player-name";
        var player = mock(Player.class);
        when(playerManagerService.getPlayers()).thenReturn(Collections.singletonList(player));
        when(player.getName()).thenReturn(name);
        when(player.getState()).thenReturn(PlayerState.UNKNOWN);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));
        var service = new PlayerInfoService(playerManagerService);

        playerListenerHolder.get().playersChanged();
        var listener = listenerHolder.get();
        listener.onStateChanged(PlayerState.ERROR);
        var result = service.getComponentDetails();

        assertEquals(1, result.size());
        assertEquals(ComponentState.ERROR, result.get(0).getState());
    }

    @Test
    void testListener_whenPlayersAreChanged_shouldInvokeListenersWithAllPlayers() {
        var name = "player-name";
        var player = mock(Player.class);
        var infoListener = mock(InfoListener.class);
        var expectedResult = SimpleComponentDetails.builder()
                .name(name)
                .state(ComponentState.UNKNOWN)
                .build();
        when(playerManagerService.getPlayers()).thenReturn(Collections.singletonList(player));
        when(player.getName()).thenReturn(name);
        when(player.getState()).thenReturn(PlayerState.UNKNOWN);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));
        var service = new PlayerInfoService(playerManagerService);

        service.addListener(infoListener);
        playerListenerHolder.get().playersChanged();

        verify(infoListener).onComponentDetailsChanged(Collections.singletonList(expectedResult));
    }
}
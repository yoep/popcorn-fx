package com.github.yoep.popcorn.ui.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerEventServiceTest {
    @Mock
    private PlayerManagerService playerService;
    @InjectMocks
    private PlayerEventService service;

    private final ObjectProperty<Player> playerProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        when(playerService.activePlayerProperty()).thenReturn(playerProperty);
    }

    @Test
    void testInit_whenInvoked_shouldListenOnPlayerChanges() {
        var player = mock(Player.class);

        service.init();
        playerProperty.set(player);

        verify(player).addListener(isA(PlayerListener.class));
    }

    @Test
    void testAddListener_whenPlayerStateIsChanged_shouldInvokeStateChangedOnTheSubscribedPlayerListener() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var listener = mock(PlayerListener.class);
        var player = mock(Player.class);
        var expectedState = PlayerState.PLAYING;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));

        service.init();
        playerProperty.set(player);
        service.addListener(listener);
        listenerHolder.get().onStateChanged(expectedState);

        verify(listener).onStateChanged(expectedState);
    }

    @Test
    void testAddListener_whenPlayerDurationIsChanged_shouldInvokeDurationChangedOnTheSubscribedPlayerListener() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var listener = mock(PlayerListener.class);
        var player = mock(Player.class);
        var expectedDuration = 200L;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));

        service.init();
        playerProperty.set(player);
        service.addListener(listener);
        listenerHolder.get().onDurationChanged(expectedDuration);

        verify(listener).onDurationChanged(expectedDuration);
    }

    @Test
    void testAddListener_whenPlayerTimeIsChanged_shouldInvokeTimeChangedOnTheSubscribedPlayerListener() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var listener = mock(PlayerListener.class);
        var player = mock(Player.class);
        var expectedTime = 150L;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));

        service.init();
        playerProperty.set(player);
        service.addListener(listener);
        listenerHolder.get().onTimeChanged(expectedTime);

        verify(listener).onTimeChanged(expectedTime);
    }

    @Test
    void testRemoveListener_whenPlayerStateIsChanged_shouldNotInvokedPlayerStateChangedOnTheListener() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var listener = mock(PlayerListener.class);
        var player = mock(Player.class);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));

        service.init();
        playerProperty.set(player);
        service.addListener(listener);
        service.removeListener(listener);
        listenerHolder.get().onStateChanged(PlayerState.BUFFERING);

        verify(listener, times(0)).onStateChanged(isA(PlayerState.class));
    }
}

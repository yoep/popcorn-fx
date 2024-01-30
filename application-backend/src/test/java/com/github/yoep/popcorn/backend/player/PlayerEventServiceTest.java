package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerChangedEvent;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerEventServiceTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private PlayerManagerService playerService;
    @InjectMocks
    private PlayerEventService service;

    @BeforeEach
    void setUp() {
    }

    @Test
    void testInit_whenInvoked_shouldListenOnPlayerChanges() {
        var oldPlayerId = "MyOldPlayer";
        var newPlayerId = "MyNewPlayer";
        var oldPlayer = mock(Player.class);
        var newPlayer = mock(Player.class);
        when(playerService.getById(oldPlayerId)).thenReturn(Optional.of(oldPlayer));
        when(playerService.getById(newPlayerId)).thenReturn(Optional.of(newPlayer));

        service.init();
        eventPublisher.publish(PlayerChangedEvent.builder()
                .source(this)
                .oldPlayerId(oldPlayerId)
                .newPlayerId(newPlayerId)
                .build());

        verify(oldPlayer).removeListener(isA(PlayerListener.class));
        verify(newPlayer).addListener(isA(PlayerListener.class));
    }

    @Test
    void testAddListener_whenPlayerStateIsChanged_shouldInvokeStateChangedOnTheSubscribedPlayerListener() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var listener = mock(PlayerListener.class);
        var playerId = "newPlayerId";
        var player = mock(Player.class);
        var expectedState = PlayerState.PLAYING;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));
        when(playerService.getById(playerId)).thenReturn(Optional.of(player));

        service.init();
        eventPublisher.publish(PlayerChangedEvent.builder()
                .source(this)
                .newPlayerId(playerId)
                .build());
        service.addListener(listener);
        listenerHolder.get().onStateChanged(expectedState);

        verify(listener).onStateChanged(expectedState);
    }

    @Test
    void testAddListener_whenPlayerDurationIsChanged_shouldInvokeDurationChangedOnTheSubscribedPlayerListener() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var listener = mock(PlayerListener.class);
        var playerId = "newPlayerId";
        var player = mock(Player.class);
        var expectedDuration = 200L;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));
        when(playerService.getById(playerId)).thenReturn(Optional.of(player));

        service.init();
        eventPublisher.publish(PlayerChangedEvent.builder()
                .source(this)
                .newPlayerId(playerId)
                .build());
        service.addListener(listener);
        listenerHolder.get().onDurationChanged(expectedDuration);

        verify(listener).onDurationChanged(expectedDuration);
    }

    @Test
    void testAddListener_whenPlayerTimeIsChanged_shouldInvokeTimeChangedOnTheSubscribedPlayerListener() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var listener = mock(PlayerListener.class);
        var playerId = "newPlayerId";
        var player = mock(Player.class);
        var expectedTime = 150L;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));
        when(playerService.getById(playerId)).thenReturn(Optional.of(player));

        service.init();
        eventPublisher.publish(PlayerChangedEvent.builder()
                .source(this)
                .newPlayerId(playerId)
                .build());
        service.addListener(listener);
        listenerHolder.get().onTimeChanged(expectedTime);

        verify(listener).onTimeChanged(expectedTime);
    }

    @Test
    void testRemoveListener_whenPlayerStateIsChanged_shouldNotInvokedPlayerStateChangedOnTheListener() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var listener = mock(PlayerListener.class);
        var playerId = "newPlayerId";
        var player = mock(Player.class);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));
        when(playerService.getById(playerId)).thenReturn(Optional.of(player));

        service.init();
        eventPublisher.publish(PlayerChangedEvent.builder()
                .source(this)
                .newPlayerId(playerId)
                .build());
        service.addListener(listener);
        service.removeListener(listener);
        listenerHolder.get().onStateChanged(PlayerState.BUFFERING);

        verify(listener, times(0)).onStateChanged(isA(PlayerState.class));
    }

    @Test
    void testOnClosePlayerEvent() {
        var player = mock(Player.class);
        when(playerService.getActivePlayer()).thenReturn(Optional.of(player));
        service.init();

        eventPublisher.publish(new ClosePlayerEvent(this, ClosePlayerEvent.Reason.USER));

        verify(player).stop();
    }
}
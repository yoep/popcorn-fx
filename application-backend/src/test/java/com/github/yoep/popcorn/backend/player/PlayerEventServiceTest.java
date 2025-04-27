package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerEventServiceTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private PlayerManagerService playerService;
    @Mock
    private Player player;
    private PlayerEventService service;

    private final AtomicReference<PlayerManagerListener> playerManagerListener = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            playerManagerListener.set(invocations.getArgument(0, PlayerManagerListener.class));
            return null;
        }).when(playerService).addListener(isA(PlayerManagerListener.class));
        lenient().when(playerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.of(player)));

        service = new PlayerEventService(playerService, eventPublisher);
    }

    @Test
    void testOnClosePlayerEvent() {
        eventPublisher.publish(new ClosePlayerEvent(this, ClosePlayerEvent.Reason.USER));

        verify(player).stop();
    }

    @Test
    void testOnPlayerTimeChanged() {
        var newTime = 120000L;
        var listener = mock(PlayerListener.class);
        service.addListener(listener);

        playerManagerListener.get().onPlayerTimeChanged(newTime);

        verify(listener).onTimeChanged(newTime);
    }

    @Test
    void testOnPlayerDurationChanged() {
        var newTime = 10L;
        var listener = mock(PlayerListener.class);
        service.addListener(listener);

        playerManagerListener.get().onPlayerDurationChanged(newTime);

        verify(listener).onDurationChanged(newTime);
    }

    @Test
    void testOnPlayerStateChanged() {
        var newState = com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.State.PLAYING;
        var listener = mock(PlayerListener.class);
        service.addListener(listener);

        playerManagerListener.get().onPlayerStateChanged(newState);

        verify(listener).onStateChanged(newState);
    }
}
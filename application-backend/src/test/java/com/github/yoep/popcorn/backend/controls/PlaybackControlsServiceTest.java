package com.github.yoep.popcorn.backend.controls;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ControlEvent;
import com.github.yoep.popcorn.backend.player.PlayerEventService;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlaybackControlsServiceTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private PlayerEventService playerEventService;
    @Mock
    private Player player;
    private PlaybackControlsService service;

    private final AtomicReference<FxCallback<ControlEvent>> subscriptionHolder = new AtomicReference<>();
    private final AtomicReference<PlayerListener> playerListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            subscriptionHolder.set(invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(ControlEvent.class)), isA(Parser.class), isA(FxCallback.class));
        doAnswer(invocations -> {
            playerListenerHolder.set(invocations.getArgument(0, PlayerListener.class));
            return null;
        }).when(playerEventService).addListener(isA(PlayerListener.class));
        lenient().when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.of(player)));

        service = new PlaybackControlsService(fxChannel, playerManagerService, playerEventService);
    }

    @Test
    void testListener_OnTimeChanged() {
        var timeValue = 12000L;
        var eventListener = playerListenerHolder.get();

        eventListener.onTimeChanged(timeValue);

        assertEquals(timeValue, service.lastKnownTime);
    }

    @Test
    void testListener_OnForward() {
        var event = ControlEvent.newBuilder()
                .setEvent(ControlEvent.Event.FORWARD)
                .build();
        var listener = (FxCallback<ControlEvent>) mock(FxCallback.class);
        playerListenerHolder.get().onTimeChanged(10000);

        service.register(listener);
        subscriptionHolder.get().callback(event);

        verify(player).seek(20000);
    }

    @Test
    void testListener_OnRewind() {
        var event = ControlEvent.newBuilder()
                .setEvent(ControlEvent.Event.REWIND)
                .build();
        var listener = (FxCallback<ControlEvent>) mock(FxCallback.class);
        playerListenerHolder.get().onTimeChanged(10000);

        service.register(listener);
        subscriptionHolder.get().callback(event);

        verify(player).seek(0);
    }
}
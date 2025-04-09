package com.github.yoep.popcorn.backend.controls;

import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.PlayerState;
import com.github.yoep.popcorn.backend.player.PlayerEventService;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
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

    private final AtomicReference<PlaybackControlCallback> callbackHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));

        service = new PlaybackControlsService(fxLib, instance, playerManagerService, playerEventService);
    }

    @Test
    void testOnTogglePlaybackEvent() throws ExecutionException, InterruptedException, TimeoutException {
        var eventFuture = new CompletableFuture<PlaybackControlEvent>();
        service.register(eventFuture::complete);
        when(player.getState()).thenReturn(PlayerState.PLAYING, PlayerState.PAUSED);
        var listener = callbackHolder.get();

        listener.callback(PlaybackControlEvent.TogglePlaybackState);
        verify(player).pause();
        assertEquals(PlaybackControlEvent.TogglePlaybackState, eventFuture.get(100, TimeUnit.MILLISECONDS));

        listener.callback(PlaybackControlEvent.TogglePlaybackState);
        verify(player).resume();
    }
}
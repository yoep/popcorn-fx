package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.player.popcorn.controllers.components.PlayerControlsComponent;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class PlaybackEventServiceTest {
    @Mock
    private PopcornPlayer player;
    @Mock
    private PlayerControlsComponent playerControls;
    @InjectMocks
    private PlaybackEventService service;

    @Test
    void testInit_whenInvoked_shouldAddListenerToPlayer() {
        service.init();

        verify(player).addListener(isA(PlayerListener.class));
    }

    @Test
    void testListener_whenStateIsUpdatedToPlaying_shouldUpdatePlaybackStateWithPlaying() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));

        service.init();
        var listener = listenerHolder.get();
        listener.onStateChanged(PlayerState.PLAYING);

        verify(playerControls).updatePlaybackState(true);
    }

    @Test
    void testListener_whenStateIsUpdatedToPaused_shouldUpdatePlaybackStateWithPaused() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));

        service.init();
        var listener = listenerHolder.get();
        listener.onStateChanged(PlayerState.PAUSED);

        verify(playerControls).updatePlaybackState(false);
    }

    @Test
    void testListener_whenDurationIsUpdated_shouldUpdatePlaybackDuration() {
        var duration = 50000L;
        var listenerHolder = new AtomicReference<PlayerListener>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));

        service.init();
        var listener = listenerHolder.get();
        listener.onDurationChanged(duration);

        verify(playerControls).updateDuration(duration);
    }

    @Test
    void testListener_whenTimeIsUpdated_shouldUpdatePlaybackTime() {
        var time = 758000L;
        var listenerHolder = new AtomicReference<PlayerListener>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));

        service.init();
        var listener = listenerHolder.get();
        listener.onTimeChanged(time);

        verify(playerControls).updateTime(time);
    }
}

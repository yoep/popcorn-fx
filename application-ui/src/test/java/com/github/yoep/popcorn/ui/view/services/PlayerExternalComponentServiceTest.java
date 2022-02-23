package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import java.util.Optional;

import static org.mockito.Mockito.*;


@ExtendWith(MockitoExtension.class)
class PlayerExternalComponentServiceTest {
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @InjectMocks
    private PlayerExternalComponentService service;

    @Test
    void testTogglePlaybackState_whenPlayerIsPaused_shouldResumePlayer() {
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(player.getState()).thenReturn(PlayerState.PAUSED);

        service.togglePlaybackState();

        verify(player).resume();
    }

    @Test
    void testTogglePlaybackState_whenPlayerIsPlaying_shouldPausePlayer() {
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(player.getState()).thenReturn(PlayerState.PLAYING);

        service.togglePlaybackState();

        verify(player).pause();
    }

    @Test
    void testClosePlayer_whenInvoked_shouldStopAndCloseThePlayer() {
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));

        service.closePlayer();

        verify(player).stop();
        verify(eventPublisher).publishEvent(new ClosePlayerEvent(service, ClosePlayerEvent.Reason.USER));
    }
}
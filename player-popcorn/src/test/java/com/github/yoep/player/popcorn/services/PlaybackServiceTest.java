package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.state.PlayerState;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class PlaybackServiceTest {
    @Mock
    private RegisterService registerService;
    @Mock
    private Player player;
    @InjectMocks
    private PlaybackService playbackService;

    @BeforeEach
    void setUp() {
        when(registerService.getPlayer()).thenReturn(player);
    }

    @Test
    void testTogglePlayerPause_whenPlayerIsPaused_shouldResumeThePlayer() {
        when(player.getState()).thenReturn(PlayerState.PAUSED);

        playbackService.togglePlayerPlaybackState();

        verify(player).resume();
    }

    @Test
    void testTogglePlayerPause_whenPlayerIsPlaying_shouldPauseThePlayer() {
        when(player.getState()).thenReturn(PlayerState.PLAYING);

        playbackService.togglePlayerPlaybackState();

        verify(player).pause();
    }

    @Test
    void testStop_whenInvoked_shouldStopThePlayerPlayback() {
        playbackService.stop();

        verify(player).stop();
    }

    @Test
    void testResume_whenInvoked_shouldResumeThePlayer() {
        playbackService.resume();

        verify(player).resume();
    }

    @Test
    void testPause_whenInvoked_shouldPauseThePlayer() {
        playbackService.pause();

        verify(player).pause();
    }

    @Test
    void testSeek_whenInvoked_shouldSeekWithinThePlayer() {
        var time = 84445000L;

        playbackService.seek(time);

        verify(player).seek(time);
    }
}

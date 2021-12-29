package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
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
    private PopcornPlayer popcornPlayer;
    @InjectMocks
    private PlaybackService playbackService;

    @Test
    void testTogglePlayerPause_whenPlayerIsPaused_shouldResumeThePlayer() {
        when(popcornPlayer.getState()).thenReturn(PlayerState.PAUSED);

        playbackService.togglePlayerPlaybackState();

        verify(popcornPlayer).resume();
    }

    @Test
    void testTogglePlayerPause_whenPlayerIsPlaying_shouldPauseThePlayer() {
        when(popcornPlayer.getState()).thenReturn(PlayerState.PLAYING);

        playbackService.togglePlayerPlaybackState();

        verify(popcornPlayer).pause();
    }

    @Test
    void testStop_whenInvoked_shouldStopThePlayerPlayback() {
        playbackService.stop();

        verify(popcornPlayer).stop();
    }

    @Test
    void testResume_whenInvoked_shouldResumeThePlayer() {
        playbackService.resume();

        verify(popcornPlayer).resume();
    }

    @Test
    void testPause_whenInvoked_shouldPauseThePlayer() {
        playbackService.pause();

        verify(popcornPlayer).pause();
    }

    @Test
    void testSeek_whenInvoked_shouldSeekWithinThePlayer() {
        var time = 84445000L;

        playbackService.seek(time);

        verify(popcornPlayer).seek(time);
    }
}

package com.github.yoep.player.popcorn;

import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.listeners.VideoListener;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PopcornPlayerTest {
    @InjectMocks
    private PopcornPlayer popcornPlayer;

    @Test
    void testGetId_whenInvoked_shouldReturnTheExpectedId() {
        var result = popcornPlayer.getId();

        assertEquals(PopcornPlayer.PLAYER_ID, result);
    }

    @Test
    void testGetName_whenInvoked_shouldReturnTheExpectedName() {
        var result = popcornPlayer.getName();

        assertEquals(PopcornPlayer.PLAYER_NAME, result);
    }

    @Test
    void testIsEmbeddedPlaybackSupported_whenInvoked_shouldReturnTrue() {
        var result = popcornPlayer.isEmbeddedPlaybackSupported();

        assertTrue(result, "Expected the popcorn player to support embedded playback");
    }

    @Test
    void testDispose_whenInvoked_shouldDisposeTheVideoPlayers() {
        popcornPlayer.dispose();
    }

    @Test
    void testInit_whenVideoPlayerIsSwitched_shouldRegisterListenerToNewVideoPlayer() {
        var oldPlayer = mock(VideoPlayer.class);
        var newPlayer = mock(VideoPlayer.class);

        popcornPlayer.updateActiveVideoPlayer(oldPlayer, newPlayer);

        verify(newPlayer).addListener(isA(VideoListener.class));
    }

    @Test
    void testInit_whenVideoPlayerIsSwitched_shouldUnregisterTheListenerFromTheOldPlayer() {
        var oldPlayer = mock(VideoPlayer.class);
        var newPlayer = mock(VideoPlayer.class);

        popcornPlayer.updateActiveVideoPlayer(oldPlayer, newPlayer);

        verify(oldPlayer).removeListener(isA(VideoListener.class));
    }
}

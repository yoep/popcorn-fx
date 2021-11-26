package com.github.yoep.player.popcorn.services;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class VideoServiceTest {
    @Mock
    private RegisterService registerService;

    @Test
    void testGetVideoPlayer_whenNoVideoPlayerIsActive_shouldReturnEmpty() {
        var service = new VideoService(Collections.emptyList(), registerService);

        var result = service.getVideoPlayer();

        assertTrue(result.isEmpty(), "Expected no video player to be active");
    }

    @Test
    void testGetVideoPlayer_whenVideoPlayerIsSwitched_shouldReturnTheActiveVideoPlayer() {
        var url = "lorem_ipsum_dolor.mp4";
        var player = mock(VideoPlayer.class);
        var videoSurface = mock(Pane.class);
        var service = new VideoService(Collections.singletonList(player), registerService);
        when(player.supports(url)).thenReturn(true);
        when(player.getVideoSurface()).thenReturn(videoSurface);

        service.switchSupportedVideoPlayer(url);
        var result = service.getVideoPlayer();

        assertTrue(result.isPresent(), "Expected an active video player to be returned");
        assertEquals(player, result.get());
    }

    @Test
    void testSwitchSupportedVideoPlayer_whenThereIsNoSupportedVideoPlayer_shouldThrowVideoPlayerException() {
        var url = "my-invalid-url.jpg";
        var player = mock(VideoPlayer.class);
        var service = new VideoService(Collections.singletonList(player), registerService);
        when(player.supports(url)).thenReturn(false);

        assertThrows(VideoPlayerException.class, () -> service.switchSupportedVideoPlayer(url));
    }

    @Test
    void testDispose_whenInvoked_shouldDisposeAllVideoPlayers() {
        var player1 = mock(VideoPlayer.class);
        var player2 = mock(VideoPlayer.class);
        var videoPlayers = asList(player1, player2);
        var service = new VideoService(videoPlayers, registerService);

        service.dispose();

        verify(player1).dispose();
        verify(player2).dispose();
    }
}

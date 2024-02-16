package com.github.yoep.player.popcorn.player;

import com.github.yoep.player.popcorn.services.VideoService;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PopcornPlayerTest {
    @Mock
    private VideoService videoService;
    @InjectMocks
    private PopcornPlayer popcornPlayer;

    private final ObjectProperty<VideoPlayback> videoPlayerProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        lenient().when(videoService.videoPlayerProperty()).thenReturn(videoPlayerProperty);
    }

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

        assertFalse(result, "Expected the popcorn player to NOT support embedded playback");
    }

    @Test
    void testGetGraphicResource_whenInvokedShouldReturnTheGraphicsNode() {
        var result = popcornPlayer.getGraphicResource();

        assertTrue(result.isPresent(), "Expected the graphics node to be present");
        assertEquals(PopcornPlayer.GRAPHIC_RESOURCE, result.get());
    }

    @Test
    void testIsEmbeddedPlaybackSupported_whenInvoked_shouldReturnFalse() {
        var result = popcornPlayer.isEmbeddedPlaybackSupported();

        assertFalse(result);
    }

    @Test
    void testPlay_whenInvoked_shouldInvokePlayOnVideoService() {
        var request = mock(PlayRequest.class);

        popcornPlayer.play(request);

        verify(videoService).onPlay(request);
    }

    @Test
    void testDispose_whenInvoked_shouldDisposeTheVideoPlayers() {
        popcornPlayer.dispose();
    }

    @Test
    void testVideoPlayerListener_whenChanged_shouldAddVideoListener() {
        var videoPlayer = mock(VideoPlayback.class);
        popcornPlayer.init();

        videoPlayerProperty.set(videoPlayer);

        verify(videoPlayer).addListener(isA(VideoListener.class));
    }
}

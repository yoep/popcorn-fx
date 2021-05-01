package com.github.yoep.player.popcorn;

import com.github.yoep.player.popcorn.services.VideoService;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.listeners.VideoListener;
import javafx.beans.property.ObjectProperty;
import javafx.beans.value.ChangeListener;
import javafx.scene.Node;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PopcornPlayerTest {
    @Mock
    private VideoService videoService;
    @Mock
    private Node embeddablePlayer;
    @Mock
    private ObjectProperty<VideoPlayer> videoPlayerProperty;

    private PopcornPlayer popcornPlayer;

    @BeforeEach
    void setUp() {
        when(videoService.videoPlayerProperty()).thenReturn(videoPlayerProperty);
        popcornPlayer = new PopcornPlayer(videoService, embeddablePlayer);
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

        assertTrue(result, "Expected the popcorn player to support embedded playback");
    }

    @Test
    void testGetEmbeddablePlayer_whenInvoked_shouldReturnTheEmbeddablePlayer() {
        var result = popcornPlayer.getEmbeddedPlayer();

        assertEquals(embeddablePlayer, result);
    }

    @Test
    void testDispose_whenInvoked_shouldDisposeTheVideoPlayers() {
        popcornPlayer.dispose();

        verify(videoService).dispose();
    }

    @Test
    void testInit_whenVideoPlayerIsSwitched_shouldRegisterListenerToNewVideoPlayer() {
        var newPlayer = mock(VideoPlayer.class);
        var listenerHolder = new AtomicReference<ChangeListener<VideoPlayer>>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, ChangeListener.class));
            return null;
        }).when(videoPlayerProperty).addListener(isA(ChangeListener.class));

        popcornPlayer = new PopcornPlayer(videoService, embeddablePlayer);
        listenerHolder.get().changed(null, null, newPlayer);

        verify(newPlayer).addListener(isA(VideoListener.class));
    }

    @Test
    void testInit_whenVideoPlayerIsSwitched_shouldUnregisterTheListenerFromTheOldPlayer() {
        var oldPlayer = mock(VideoPlayer.class);
        var newPlayer = mock(VideoPlayer.class);
        var listenerHolder = new AtomicReference<ChangeListener<VideoPlayer>>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, ChangeListener.class));
            return null;
        }).when(videoPlayerProperty).addListener(isA(ChangeListener.class));

        popcornPlayer = new PopcornPlayer(videoService, embeddablePlayer);
        listenerHolder.get().changed(null, oldPlayer, newPlayer);

        verify(oldPlayer).removeListener(isA(VideoListener.class));
    }
}

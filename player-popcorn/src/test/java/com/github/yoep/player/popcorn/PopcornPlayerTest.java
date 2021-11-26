package com.github.yoep.player.popcorn;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.listeners.VideoListener;
import javafx.beans.property.ObjectProperty;
import javafx.scene.Node;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PopcornPlayerTest {
    @Mock
    private Node embeddablePlayer;

    @Test
    void testGetId_whenInvoked_shouldReturnTheExpectedId() {
        var listeners = Collections.<PlaybackListener>emptyList();
        var popcornPlayer = new PopcornPlayer(listeners, embeddablePlayer);
        var result = popcornPlayer.getId();

        assertEquals(PopcornPlayer.PLAYER_ID, result);
    }

    @Test
    void testGetName_whenInvoked_shouldReturnTheExpectedName() {
        var listeners = Collections.<PlaybackListener>emptyList();
        var popcornPlayer = new PopcornPlayer(listeners, embeddablePlayer);

        var result = popcornPlayer.getName();

        assertEquals(PopcornPlayer.PLAYER_NAME, result);
    }

    @Test
    void testIsEmbeddedPlaybackSupported_whenInvoked_shouldReturnTrue() {
        var listeners = Collections.<PlaybackListener>emptyList();
        var popcornPlayer = new PopcornPlayer(listeners, embeddablePlayer);

        var result = popcornPlayer.isEmbeddedPlaybackSupported();

        assertTrue(result, "Expected the popcorn player to support embedded playback");
    }

    @Test
    void testGetEmbeddablePlayer_whenInvoked_shouldReturnTheEmbeddablePlayer() {
        var listeners = Collections.<PlaybackListener>emptyList();
        var popcornPlayer = new PopcornPlayer(listeners, embeddablePlayer);

        var result = popcornPlayer.getEmbeddedPlayer();

        assertEquals(embeddablePlayer, result);
    }

    @Test
    void testDispose_whenInvoked_shouldDisposeTheVideoPlayers() {
        var listeners = Collections.<PlaybackListener>emptyList();
        var popcornPlayer = new PopcornPlayer(listeners, embeddablePlayer);

        popcornPlayer.dispose();
    }

    @Test
    void testInit_whenVideoPlayerIsSwitched_shouldRegisterListenerToNewVideoPlayer() {
        var oldPlayer = mock(VideoPlayer.class);
        var newPlayer = mock(VideoPlayer.class);
        var listeners = Collections.<PlaybackListener>emptyList();
        var popcornPlayer = new PopcornPlayer(listeners, embeddablePlayer);

        popcornPlayer.updateActiveVideoPlayer(oldPlayer, newPlayer);

        verify(newPlayer).addListener(isA(VideoListener.class));
    }

    @Test
    void testInit_whenVideoPlayerIsSwitched_shouldUnregisterTheListenerFromTheOldPlayer() {
        var oldPlayer = mock(VideoPlayer.class);
        var newPlayer = mock(VideoPlayer.class);
        var listeners = Collections.<PlaybackListener>emptyList();
        var popcornPlayer = new PopcornPlayer(listeners, embeddablePlayer);

        popcornPlayer.updateActiveVideoPlayer(oldPlayer, newPlayer);

        verify(oldPlayer).removeListener(isA(VideoListener.class));
    }
}

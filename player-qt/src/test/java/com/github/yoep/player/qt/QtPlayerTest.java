package com.github.yoep.player.qt;

import com.github.yoep.player.qt.player.PopcornPlayer;
import com.github.yoep.player.qt.player.PopcornPlayerEventListener;
import com.github.yoep.player.qt.player.PopcornPlayerState;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Mockito;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class QtPlayerTest {
    @Mock
    private PopcornPlayer popcornPlayer;

    private QtPlayer player;

    private final AtomicReference<PopcornPlayerEventListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PopcornPlayerEventListener.class));
            return null;
        }).when(popcornPlayer).addListener(isA(PopcornPlayerEventListener.class));

        player = new QtPlayer(popcornPlayer);
    }

    @Test
    void testGetId_whenInvoked_shouldReturnTheId() {
        var result = player.getId();

        assertEquals(QtPlayer.ID, result);
    }

    @Test
    void testGetName_whenInvoked_shouldReturnTheExpectedName() {
        var result = player.getName();

        assertEquals(QtPlayer.NAME, result);
    }

    @Test
    void testGetGraphicsResource_whenInvoked_shouldReturnTheExpectedName() {
        var result = player.getGraphicResource();

        assertTrue(result.isPresent(), "Expected a graphics resource to be present");
        assertEquals(QtPlayer.GRAPHIC_RESOURCE, result.get());
    }

    @Test
    void testIsEmbeddedPlaybackSupported_whenInvoked_shouldReturnFalse() {
        var result = player.isEmbeddedPlaybackSupported();

        assertFalse(result, "Expected the QT player to not support embedded playback");
    }

    @Test
    void testDispose_whenInvoked_shouldReleaseTheNativePlayer() {
        player.dispose();

        verify(popcornPlayer).release();
    }

    @Test
    void testPlay_whenInitialized_shouldShowAndPlayRequestUrl() {
        var url = "http://localhost/my-video-url.mp4";
        var request = SimplePlayRequest.builder()
                .url(url)
                .build();

        player.play(request);

        verify(popcornPlayer).show();
        verify(popcornPlayer).play(url);
    }

    @Test
    void testPlayerListener_whenStateIsChanged_shouldStoreAndInvokeListeners() {
        var playerListener = Mockito.mock(PlayerListener.class);
        var expectedState = PlayerState.BUFFERING;
        var listener = listenerHolder.get();
        player.addListener(playerListener);

        listener.onStateChanged(PopcornPlayerState.BUFFERING);
        var result = player.getState();

        assertEquals(expectedState, result);
        verify(playerListener).onStateChanged(expectedState);
    }

    @Test
    void testPlayerListener_whenStateIsPlaying_shouldInvokeListenersWithCorrectState() {
        var playerListener = Mockito.mock(PlayerListener.class);
        var expectedState = PlayerState.PLAYING;
        var listener = listenerHolder.get();
        player.addListener(playerListener);

        listener.onStateChanged(PopcornPlayerState.PLAYING);

        verify(playerListener).onStateChanged(expectedState);
    }

    @Test
    void testPlayerListener_whenStateIsPaused_shouldInvokeListenersWithCorrectState() {
        var playerListener = Mockito.mock(PlayerListener.class);
        var expectedState = PlayerState.PAUSED;
        var listener = listenerHolder.get();
        player.addListener(playerListener);

        listener.onStateChanged(PopcornPlayerState.PAUSED);

        verify(playerListener).onStateChanged(expectedState);
    }

    @Test
    void testPlayerListener_whenStateIsStopped_shouldInvokeListenersWithCorrectState() {
        var playerListener = Mockito.mock(PlayerListener.class);
        var expectedState = PlayerState.STOPPED;
        var listener = listenerHolder.get();
        player.addListener(playerListener);

        listener.onStateChanged(PopcornPlayerState.STOPPED);

        verify(playerListener).onStateChanged(expectedState);
    }

    @Test
    void testPlayerListener_whenTimeIsChanged_shouldInvokeListeners() {
        var playerListener = Mockito.mock(PlayerListener.class);
        var expectedTime = 10000;
        var listener = listenerHolder.get();
        player.addListener(playerListener);

        listener.onTimeChanged(expectedTime);

        verify(playerListener).onTimeChanged(expectedTime);
    }

    @Test
    void testResume_whenInvoked_shouldResumePlayer() {
        player.resume();

        verify(popcornPlayer).resume();
    }

    @Test
    void testPause_whenInvoked_shouldPausePlayer() {
        player.pause();

        verify(popcornPlayer).pause();
    }

    @Test
    void testStop_whenInvoked_shouldStopThePlayer() {
        player.stop();

        verify(popcornPlayer).stop();
    }

    @Test
    void testSeek_whenTimeIsGiven_shouldSeekTheTimeInThePlayer() {
        var time = 189900;

        player.seek(time);

        verify(popcornPlayer).seek(time);
    }

    @Test
    void testPlayerListener_whenDurationIsChanged_shouldInvokeListeners() {
        var playerListener = Mockito.mock(PlayerListener.class);
        var expectedDuration = 20000;
        var listener = listenerHolder.get();
        player.addListener(playerListener);

        listener.onDurationChanged(expectedDuration);

        verify(playerListener).onDurationChanged(expectedDuration);
    }
}
package com.github.yoep.player.vlc;

import com.github.yoep.player.vlc.model.VlcState;
import com.github.yoep.player.vlc.services.VlcPlayerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class VlcPlayerTest {
    @Mock
    private VlcPlayerService service;

    private VlcPlayer player;

    private final AtomicReference<VlcListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, VlcListener.class));
            return null;
        }).when(service).addListener(isA(VlcListener.class));

        player = new VlcPlayer(service);
    }

    @Test
    void testGetId_whenInvoked_shouldReturnTheExpectedId() {
        var result = player.getId();

        assertEquals(VlcPlayer.IDENTIFIER, result);
    }

    @Test
    void testGetName_whenInvoked_shouldReturnTheExpectedId() {
        var result = player.getName();

        assertEquals(VlcPlayer.IDENTIFIER, result);
    }

    @Test
    void testGetDescription_whenInvoked_shouldReturnTheExpectedId() {
        var result = player.getDescription();

        assertEquals(VlcPlayer.DESCRIPTION, result);
    }

    @Test
    void testGetGraphicsResource_whenInvoked_shouldReturnTheExpectedResult() {
        var expectedResult = Optional.of(VlcPlayer.GRAPHIC_RESOURCE);

        var result = player.getGraphicResource();

        assertEquals(expectedResult, result);
    }

    @Test
    void testIsEmbeddedPlaybackSupported_whenInvoked_shouldReturnFalse() {
        var result = player.isEmbeddedPlaybackSupported();

        assertFalse(result);
    }

    @Test
    void testDispose_whenInvoked_shouldStopThePlayback() {
        player.dispose();

        verify(service).stop();
    }

    @Test
    void testPlay_whenProcessFailedToLaunch_shouldUpdateStateToError() {
        var request = SimplePlayRequest.builder()
                .url("my-video-url")
                .build();
        when(service.play(request)).thenReturn(false);

        player.play(request);

        assertEquals(PlayerState.ERROR, player.getState());
    }

    @Test
    void testResume_whenInvoked_shouldResumeThePlayer() {
        player.resume();

        verify(service).executeCommand(VlcPlayer.COMMAND_PLAY_PAUSE);
    }

    @Test
    void testPause_whenInvoked_shouldPauseThePlayer() {
        player.pause();

        verify(service).executeCommand(VlcPlayer.COMMAND_PLAY_PAUSE);
    }

    @Test
    void testSeek_whenTimeIsGiven_shouldSeekThePlayerWithTheExpectedTime() {
        var time = 60000L;

        player.seek(time);

        verify(service).executeCommand(VlcPlayer.COMMAND_SEEK, "60");
    }

    @Test
    void testStop_whenInvoked_shouldStopThePlayback() {
        player.stop();

        verify(service).executeCommand(VlcPlayer.COMMAND_STOP);
    }

    @Test
    void testVolume_whenVolumeIsGiven_shouldSetThePlayerVolume() {
        var volume = 75;

        player.volume(volume);

        verify(service).executeCommand(VlcPlayer.COMMAND_VOLUME, "192");
    }

    @Test
    void testListeners_whenTimeIsChanged_shouldInvokeListeners() {
        var playerListener = mock(PlayerListener.class);
        player.addListener(playerListener);

        var listener = listenerHolder.get();
        listener.onTimeChanged(25L);

        verify(playerListener).onTimeChanged(25000L);
    }

    @Test
    void testListeners_whenDurationIsChanged_shouldInvokeListeners() {
        var playerListener = mock(PlayerListener.class);
        player.addListener(playerListener);

        var listener = listenerHolder.get();
        listener.onDurationChanged(102L);

        verify(playerListener).onDurationChanged(102000L);
    }

    @Test
    void testListeners_whenStateIsChanged_shouldInvokeListeners() {
        var playerListener = mock(PlayerListener.class);
        player.addListener(playerListener);

        var listener = listenerHolder.get();
        listener.onStateChanged(VlcState.PAUSED);

        verify(playerListener).onStateChanged(PlayerState.PAUSED);
    }
}
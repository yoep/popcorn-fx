package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.player.model.MediaPlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerControlsServiceTest {
    @Mock
    private PopcornPlayer player;
    @Mock
    private ScreenService screenService;
    @Mock
    private VideoService videoService;
    @Mock
    private PlayerControlsListener listener;
    @InjectMocks
    private PlayerControlsService service;

    private final AtomicReference<PlayerListener> playerListenerHolder = new AtomicReference<>();
    private final AtomicReference<PlaybackListener> playbackListenerHolder = new AtomicReference<>();
    private final BooleanProperty fullscreenProperty = new SimpleBooleanProperty();

    @BeforeEach
    void setUp() {
        lenient().when(screenService.fullscreenProperty()).thenReturn(fullscreenProperty);
        lenient().doAnswer(invocation -> {
            playerListenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));
        lenient().doAnswer(invocation -> {
            playbackListenerHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));

        service.addListener(listener);
    }

    @Test
    void testToggleFullscreen_whenInvoked_shouldToggleFullscreenOnTheScreenService() {
        service.toggleFullscreen();

        verify(screenService).toggleFullscreen();
    }

    @Test
    void testTogglePlayerPlaybackState_whenPlayerIsPlaying_shouldPausePlayer() {
        when(player.getState()).thenReturn(PlayerState.PLAYING);

        service.togglePlayerPlaybackState();

        verify(player).pause();
    }

    @Test
    void testTogglePlayerPlaybackState_whenPlayerIsPaused_shouldResumePlayer() {
        when(player.getState()).thenReturn(PlayerState.PAUSED);

        service.togglePlayerPlaybackState();

        verify(player).resume();
    }

    @Test
    void testOnSeekChanging_whenIsSeeking_shouldPauseThePlayer() {
        service.onSeekChanging(true);

        verify(player).pause();
    }

    @Test
    void testOnSeekChanging_whenStoppedSeeking_shouldResumeThePlayer() {
        service.onSeekChanging(false);

        verify(player).resume();
    }

    @Test
    void testSeek_whenTimeIsGiven_shouldSeekTheTimeInThePlayer() {
        var time = 10078;

        service.seek(time);

        verify(player).seek(time);
    }

    @Test
    void testOnFullScreenProperty_whenFullscreenIsChanged_shouldInvokedListeners() {
        var expectedState = true;
        service.init();

        fullscreenProperty.set(expectedState);

        verify(listener).onFullscreenStateChanged(expectedState);
    }

    @Test
    void testPlayerListener_whenPlayerStateChanged_shouldInvokeListeners() {
        var state = PlayerState.STOPPED;
        service.init();

        playerListenerHolder.get().onStateChanged(state);

        verify(listener).onPlayerStateChanged(state);
    }

    @Test
    void testPlayerListener_whenPlayerTimeChanged_shouldInvokeListeners() {
        var value = 123987777;
        service.init();

        playerListenerHolder.get().onTimeChanged(value);

        verify(listener).onPlayerTimeChanged(value);
    }

    @Test
    void testPlayerListener_whenPlayerDurationChanged_shouldInvokeListeners() {
        var value = 160000;
        service.init();

        playerListenerHolder.get().onDurationChanged(value);

        verify(listener).onPlayerDurationChanged(value);
    }

    @Test
    void testPlaybackListener_whenRequestIsMediaPlayback_shouldEnableSubtitles() {
        var request = MediaPlayRequest.mediaBuilder()
                .title("lorem")
                .thumb("ipsum")
                .build();
        service.init();

        playbackListenerHolder.get().onPlay(request);

        verify(listener).onSubtitleStateChanged(true);
    }

    @Test
    void testPlaybackListener_whenRequestIsSimplePlayback_shouldDisableSubtitles() {
        var request = SimplePlayRequest.builder()
                .title("lorem")
                .thumb("ipsum")
                .build();
        service.init();

        playbackListenerHolder.get().onPlay(request);

        verify(listener).onSubtitleStateChanged(false);
    }
}
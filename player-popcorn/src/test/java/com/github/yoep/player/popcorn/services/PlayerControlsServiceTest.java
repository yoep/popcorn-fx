package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.Handle;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
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
    @Mock
    private TorrentService torrentService;

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

        service = new PlayerControlsService(player, screenService, videoService, torrentService);
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

        fullscreenProperty.set(expectedState);

        verify(listener).onFullscreenStateChanged(expectedState);
    }

    @Test
    void testPlayerListener_whenPlayerStateChanged_shouldInvokeListeners() {
        var state = PlayerState.STOPPED;

        playerListenerHolder.get().onStateChanged(state);

        verify(listener).onPlayerStateChanged(state);
    }

    @Test
    void testPlayerListener_whenPlayerTimeChanged_shouldInvokeListeners() {
        var value = 123987777;

        playerListenerHolder.get().onTimeChanged(value);

        verify(listener).onPlayerTimeChanged(value);
    }

    @Test
    void testPlayerListener_whenPlayerDurationChanged_shouldInvokeListeners() {
        var value = 160000;

        playerListenerHolder.get().onDurationChanged(value);

        verify(listener).onPlayerDurationChanged(value);
    }

    @Test
    void testPlaybackListener_whenRequestIsMediaPlayback_shouldEnableSubtitles() {
        var request = mock(PlayRequest.class);
        when(request.isSubtitlesEnabled()).thenReturn(true);

        playbackListenerHolder.get().onPlay(request);

        verify(listener).onSubtitleStateChanged(true);
    }

    @Test
    void testPlaybackListener_whenRequestIsSimplePlayback_shouldDisableSubtitles() {
        var request = mock(PlayRequest.class);

        playbackListenerHolder.get().onPlay(request);

        verify(listener).onSubtitleStateChanged(false);
    }

    @Test
    void testPlaybackListener_whenRequestIsStreamRequest_shouldInvokeDownloadStatusChanged() {
        var downloadStatus = mock(DownloadStatus.class);
        var listenerHolder = new AtomicReference<TorrentListener>();
        var request = mock(PlayRequest.class);
        when(request.getStreamHandle()).thenReturn(Optional.of(new Handle(222L)));
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(1, TorrentListener.class));
            return null;
        }).when(torrentService).addListener(isA(Handle.class), isA(TorrentListener.class));

        when(request.getStreamHandle()).thenReturn(Optional.of(new Handle(24L)));

        playbackListenerHolder.get().onPlay(request);
        listenerHolder.get().onDownloadStatus(downloadStatus);

        verify(listener).onDownloadStatusChanged(downloadStatus);
    }
}
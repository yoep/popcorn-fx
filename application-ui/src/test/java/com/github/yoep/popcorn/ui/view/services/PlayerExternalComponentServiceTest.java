package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.loader.LoaderListener;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import com.github.yoep.popcorn.backend.loader.LoadingProgress;
import com.github.yoep.popcorn.backend.loader.LoadingStartedEventC;
import com.github.yoep.popcorn.backend.player.PlayerManagerEvent;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.Mockito.*;


@ExtendWith(MockitoExtension.class)
class PlayerExternalComponentServiceTest {
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private LoaderService loaderService;
    @Mock
    private TorrentService torrentService;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @InjectMocks
    private PlayerExternalComponentService service;

    private final AtomicReference<PlayerManagerListener> playerListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            playerListenerHolder.set(invocation.getArgument(0, PlayerManagerListener.class));
            return null;
        }).when(playerManagerService).addListener(isA(PlayerManagerListener.class));
    }

    @Test
    void testListener_whenDurationIsChanged_shouldInvokeListeners() {
        var duration = 840000L;
        var playerListener = mock(PlayerExternalListener.class);
        service.init();
        service.addListener(playerListener);

        var listener = playerListenerHolder.get();
        listener.onDurationChanged(duration);

        verify(playerListener).onDurationChanged(duration);
    }

    @Test
    void testListener_whenTimeIsChanged_shouldInvokeListeners() {
        var time = 10000L;
        var playerListener = mock(PlayerExternalListener.class);
        service.init();
        service.addListener(playerListener);

        var listener = playerListenerHolder.get();
        listener.onTimeChanged(time);

        verify(playerListener).onTimeChanged(time);
    }

    @Test
    void testListener_whenStateIsChanged_shouldInvokeListeners() {
        var state = PlayerState.PLAYING;
        var playerListener = mock(PlayerExternalListener.class);
        service.init();
        service.addListener(playerListener);

        var listener = playerListenerHolder.get();
        listener.onStateChanged(state);

        verify(playerListener).onStateChanged(state);
    }

    @Test
    void testTogglePlaybackState_whenPlayerIsPaused_shouldResumePlayer() {
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(player.getState()).thenReturn(PlayerState.PAUSED);

        service.togglePlaybackState();

        verify(player).resume();
    }

    @Test
    void testTogglePlaybackState_whenPlayerIsPlaying_shouldPausePlayer() {
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(player.getState()).thenReturn(PlayerState.PLAYING);

        service.togglePlaybackState();

        verify(player).pause();
    }

    @Test
    void testClosePlayer_whenInvoked_shouldStopAndCloseThePlayer() {
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));

        service.closePlayer();

        verify(player).stop();
        verify(eventPublisher).publishEvent(new ClosePlayerEvent(service, ClosePlayerEvent.Reason.USER));
    }

    @Test
    void testGoBack_whenInvoked_shouldGoBackInTime() {
        var time = 20000L;
        var player = mock(Player.class);
        var expectedTime = time - PlayerExternalComponentService.TIME_STEP_OFFSET;
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        service.init();
        var listener = playerListenerHolder.get();
        listener.onTimeChanged(time);

        service.goBack();

        verify(player).seek(expectedTime);
    }

    @Test
    void testGoForward_whenInvoked_shouldGoForwardInTime() {
        var time = 20000L;
        var player = mock(Player.class);
        var expectedTime = time + PlayerExternalComponentService.TIME_STEP_OFFSET;
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        service.init();
        var listener = playerListenerHolder.get();
        listener.onTimeChanged(time);

        service.goForward();

        verify(player).seek(expectedTime);
    }

    @Test
    void testOnPlayerTorrent_whenDownloadStatusIsChanged_shouldInvokedListeners() {
        var listenerHolder = new AtomicReference<LoaderListener>();
        var downloadStatus = mock(LoadingProgress.class);
        var playerListener = mock(PlayerExternalListener.class);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, LoaderListener.class));
            return null;
        }).when(loaderService).addListener(isA(LoaderListener.class));
        service.init();
        service.addListener(playerListener);

        var listener = listenerHolder.get();
        listener.onProgressChanged(downloadStatus);

        verify(playerListener).onDownloadStatus(downloadStatus);
    }

    @Test
    void testOnPlayerTorrent_whenEventIsMediaEvent_shouldUpdateMedia() {
        var listenerHolder = new AtomicReference<PlayerManagerEvent>();
        var title = "Lorem ipsum";
        var playerListener = mock(PlayerExternalListener.class);
        var startedEvent = mock(LoadingStartedEventC.class);
        when(startedEvent.getTitle()).thenReturn(title);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, LoaderListener.class));
            return null;
        }).when(loaderService).addListener(isA(LoaderListener.class));
        service.init();
        service.addListener(playerListener);

        var listener = listenerHolder.get();
        listener.onLoadingStarted(startedEvent);

        verify(playerListener).onRequestChanged(title);
    }
}
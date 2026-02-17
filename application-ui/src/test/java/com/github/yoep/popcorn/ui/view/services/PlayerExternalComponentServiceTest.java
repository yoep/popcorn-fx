package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ServerStream;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Stream;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.backend.stream.IStreamServer;
import com.github.yoep.popcorn.backend.stream.StreamListener;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerExternalComponentServiceTest {
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private IStreamServer streamServer;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    private PlayerExternalComponentService service;

    private final AtomicReference<PlayerManagerListener> playerListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            playerListenerHolder.set(invocation.getArgument(0, PlayerManagerListener.class));
            return null;
        }).when(playerManagerService).addListener(isA(PlayerManagerListener.class));

        service = new PlayerExternalComponentService(playerManagerService, eventPublisher, streamServer);
    }

    @Test
    void testListener_whenDurationIsChanged_shouldInvokeListeners() {
        var duration = 840000L;
        var playerListener = mock(PlayerExternalListener.class);
        service.addListener(playerListener);

        var listener = playerListenerHolder.get();
        listener.onPlayerDurationChanged(duration);

        verify(playerListener).onDurationChanged(duration);
    }

    @Test
    void testListener_whenTimeIsChanged_shouldInvokeListeners() {
        var time = 10000L;
        var playerListener = mock(PlayerExternalListener.class);
        service.addListener(playerListener);

        var listener = playerListenerHolder.get();
        listener.onPlayerTimeChanged(time);

        verify(playerListener).onTimeChanged(time);
    }

    @Test
    void testListener_whenStateIsChanged_shouldInvokeListeners() {
        var state = com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.State.PLAYING;
        var playerListener = mock(PlayerExternalListener.class);
        service.addListener(playerListener);

        var listener = playerListenerHolder.get();
        listener.onPlayerStateChanged(state);

        verify(playerListener).onStateChanged(state);
    }

    @Test
    void testTogglePlaybackState_whenPlayerIsPaused_shouldResumePlayer() {
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.of(player)));
        when(player.getState()).thenReturn(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.State.PAUSED);

        service.togglePlaybackState();

        verify(player).resume();
    }

    @Test
    void testTogglePlaybackState_whenPlayerIsPlaying_shouldPausePlayer() {
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.of(player)));
        when(player.getState()).thenReturn(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.State.PLAYING);

        service.togglePlaybackState();

        verify(player).pause();
    }

    @Test
    void testClosePlayer_whenInvoked_shouldStopAndCloseThePlayer() {
        service.closePlayer();

        verify(eventPublisher).publishEvent(new ClosePlayerEvent(service, ClosePlayerEvent.Reason.USER));
    }

    @Test
    void testGoBack_whenInvoked_shouldGoBackInTime() {
        var time = 20000L;
        var player = mock(Player.class);
        var expectedTime = time - PlayerExternalComponentService.TIME_STEP_OFFSET;
        when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.of(player)));
        var listener = playerListenerHolder.get();
        listener.onPlayerTimeChanged(time);

        service.goBack();

        verify(player).seek(expectedTime);
    }

    @Test
    void testGoForward_whenInvoked_shouldGoForwardInTime() {
        var time = 20000L;
        var player = mock(Player.class);
        var expectedTime = time + PlayerExternalComponentService.TIME_STEP_OFFSET;
        when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.of(player)));
        var listener = playerListenerHolder.get();
        listener.onPlayerTimeChanged(time);

        service.goForward();

        verify(player).seek(expectedTime);
    }

    @Test
    void testOnPlayerTorrent_whenDownloadStatusIsChanged_shouldInvokedListeners() {
        var streamListenerHolder = new AtomicReference<StreamListener>();
        var playerListener = mock(PlayerExternalListener.class);
        var filename = "test-file.mp4";
        var request = com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.PlayRequest.newBuilder()
                .setStream(ServerStream.newBuilder()
                        .setFilename(filename)
                        .build())
                .build();
        doAnswer(invocation -> {
            streamListenerHolder.set(invocation.getArgument(1, StreamListener.class));
            return null;
        }).when(streamServer).addListener(isA(String.class), isA(StreamListener.class));
        service.addListener(playerListener);

        var listener = playerListenerHolder.get();
        listener.onPlayerPlaybackChanged(request);

        var streamListener = streamListenerHolder.get();
        streamListener.onStatsChanged(Stream.StreamStats.newBuilder()
                .setProgress(0.5f)
                .build());

        verify(playerListener).onDownloadStatus(isA(DownloadStatus.class));
        verify(streamServer).addListener(filename, streamListener);
    }

    @Test
    void testOnPlayerPlaybackChanged() {
        var playerListener = mock(PlayerExternalListener.class);
        var filename = "test-file.mp4";
        var request = com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.PlayRequest.newBuilder()
                .setStream(ServerStream.newBuilder()
                        .setFilename(filename)
                        .build())
                .build();
        service.addListener(playerListener);

        var listener = playerListenerHolder.get();
        listener.onPlayerPlaybackChanged(request);

        verify(playerListener).onRequestChanged(request);
        verify(streamServer).addListener(eq(filename), isA(StreamListener.class));
    }
}
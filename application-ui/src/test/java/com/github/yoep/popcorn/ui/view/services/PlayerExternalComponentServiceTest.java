package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayTorrentEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.player.PlayerEventService;
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
    private PlayerEventService playerEventService;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @InjectMocks
    private PlayerExternalComponentService service;

    private final AtomicReference<PlayerListener> playerListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            playerListenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(playerEventService).addListener(isA(PlayerListener.class));
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
    void testOnPlayerTorrent_whenEventIsTorrentEvent_shouldUpdateTitle() {
        var title = "Lorem ipsum";
        var event = PlayTorrentEvent.playTorrentBuilder()
                .source(this)
                .url("test-url")
                .title(title)
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .build();
        var playerListener = mock(PlayerExternalListener.class);
        service.init();
        service.addListener(playerListener);

        eventPublisher.publish(event);

        verify(playerListener).onTitleChanged(title);
    }

    @Test
    void testOnPlayerTorrent_whenDownloadStatusIsChanged_shouldInvokedListeners() {
        var torrent = mock(Torrent.class);
        var torrentListenerHolder = new AtomicReference<TorrentListener>();
        var downloadStatus = mock(DownloadStatus.class);
        var event = PlayTorrentEvent.playTorrentBuilder()
                .source(this)
                .url("test-url")
                .title("Lorem ipsum")
                .torrent(torrent)
                .torrentStream(mock(TorrentStream.class))
                .build();
        var playerListener = mock(PlayerExternalListener.class);
        doAnswer(invocation -> {
            torrentListenerHolder.set(invocation.getArgument(0, TorrentListener.class));
            return null;
        }).when(torrent).addListener(isA(TorrentListener.class));
        service.init();
        service.addListener(playerListener);

        eventPublisher.publish(event);
        var torrentListener = torrentListenerHolder.get();
        torrentListener.onDownloadStatus(downloadStatus);

        verify(playerListener).onDownloadStatus(downloadStatus);
    }

    @Test
    void testOnPlayerTorrent_whenEventIsMediaEvent_shouldUpdateMedia() {
        var title = "Lorem ipsum";
        var media = MovieDetails.builder()
                .images(Images.builder().build())
                .build();
        var event = PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("test-url")
                .title(title)
                .media(media)
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .build();
        var playerListener = mock(PlayerExternalListener.class);
        service.init();
        service.addListener(playerListener);

        eventPublisher.publish(event);

        verify(playerListener).onTitleChanged(title);
        verify(playerListener).onMediaChanged(media);
    }
}
package com.github.yoep.popcorn.ui.player;

import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.playnext.PlayNextService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerStopServiceTest {
    @Mock
    private PlayerEventService playerEventService;
    @Mock
    private TorrentStreamService torrentStreamService;
    @Mock
    private PlayNextService playNextService;
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @InjectMocks
    private PlayerStopService service;

    @Test
    void testInit_whenInvoked_shouldSubscribeToThePlayerEvents() {
        service.init();

        verify(playerEventService).addListener(isA(PlayerListener.class));
    }

    @Test
    void testOnPlayerStopped_whenIsEndOfVideoAndNextEpisodeIsNotPresent_shouldPublishClosePlayerEvent() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var videoLength = 1000L;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(playerEventService).addListener(isA(PlayerListener.class));
        when(playNextService.getNextEpisode()).thenReturn(Optional.empty());
        service.init();
        service.onPlayMedia(PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("my-movie-url")
                .title("my-title-url")
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .media(Movie.builder()
                        .images(new Images())
                        .build())
                .build());

        var playerListener = listenerHolder.get();
        playerListener.onDurationChanged(videoLength);
        playerListener.onTimeChanged(videoLength);
        playerListener.onStateChanged(PlayerState.STOPPED);

        verify(torrentStreamService).stopAllStreams();
        verify(eventPublisher).publishEvent(new com.github.yoep.popcorn.backend.events.ClosePlayerEvent(service,
                com.github.yoep.popcorn.backend.events.ClosePlayerEvent.Reason.END_OF_VIDEO));
    }

    @Test
    void testOnPlayerStopped_whenIsEndOfVideoAndNextEpisodeIsPresent_shouldNotPublishClosePlayerEvent() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        var videoLength = 1000L;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(playerEventService).addListener(isA(PlayerListener.class));
        when(playNextService.getNextEpisode()).thenReturn(Optional.of(mock(PlayNextService.NextEpisode.class)));
        service.init();
        service.onPlayMedia(PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("my-movie-url")
                .title("my-title-url")
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .media(Movie.builder()
                        .images(new Images())
                        .build())
                .build());

        var playerListener = listenerHolder.get();
        playerListener.onDurationChanged(videoLength);
        playerListener.onTimeChanged(videoLength);
        playerListener.onStateChanged(PlayerState.STOPPED);

        verify(torrentStreamService).stopAllStreams();
        verify(eventPublisher, times(0)).publishEvent(new com.github.yoep.popcorn.backend.events.ClosePlayerEvent(service,
                com.github.yoep.popcorn.backend.events.ClosePlayerEvent.Reason.END_OF_VIDEO));
    }

    @Test
    void testInit_whenPlayerIsStoppedAndDurationIsZero_shouldNotPublishClosePlayerEvent() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(playerEventService).addListener(isA(PlayerListener.class));

        service.init();
        var playerListener = listenerHolder.get();
        playerListener.onDurationChanged(0L);
        playerListener.onStateChanged(PlayerState.STOPPED);

        verify(eventPublisher, times(0)).publishEvent(isA(ClosePlayerEvent.class));
    }

    @Test
    void testInit_whenPlayerIsStopped_shouldPublishPlayerStoppedEventWithLastKnownTimeAndDuration() {
        var duration = 2000L;
        var time = 500L;
        var listenerHolder = new AtomicReference<PlayerListener>();
        var eventHolder = new AtomicReference<PlayerStoppedEvent>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(playerEventService).addListener(isA(PlayerListener.class));
        doAnswer(invocation -> {
            eventHolder.set(invocation.getArgument(0, PlayerStoppedEvent.class));
            return null;
        }).when(eventPublisher).publishEvent(isA(PlayerStoppedEvent.class));

        service.init();
        var playerListener = listenerHolder.get();
        playerListener.onDurationChanged(duration);
        playerListener.onTimeChanged(time);
        playerListener.onStateChanged(PlayerState.STOPPED);

        verify(eventPublisher).publishEvent(isA(PlayerStoppedEvent.class));
        var stoppedEvent = eventHolder.get();
        assertEquals(duration, stoppedEvent.getDuration());
        assertEquals(time, stoppedEvent.getTime());
    }

    @Test
    void testOnPlayMedia_whenPlayerIsStopped_shouldAddMediaDataToStoppedEvent() {
        var media = mock(Media.class);
        var quality = "720p";
        var url = "my-video-url";
        var event = mock(PlayMediaEvent.class);
        var listenerHolder = new AtomicReference<PlayerListener>();
        var eventHolder = new AtomicReference<PlayerStoppedEvent>();
        when(event.getMedia()).thenReturn(media);
        when(event.getUrl()).thenReturn(url);
        when(event.getQuality()).thenReturn(quality);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(playerEventService).addListener(isA(PlayerListener.class));
        doAnswer(invocation -> {
            eventHolder.set(invocation.getArgument(0, PlayerStoppedEvent.class));
            return null;
        }).when(eventPublisher).publishEvent(isA(PlayerStoppedEvent.class));

        service.init();
        service.onPlayMedia(event);
        var playerListener = listenerHolder.get();
        playerListener.onTimeChanged(1000L);
        playerListener.onDurationChanged(5000L);
        playerListener.onStateChanged(PlayerState.STOPPED);

        verify(eventPublisher).publishEvent(isA(PlayerStoppedEvent.class));
        var result = eventHolder.get();
        assertEquals(media, result.getMedia().get());
        assertEquals(url, result.getUrl());
        assertEquals(quality, result.getQuality().get());
    }
}

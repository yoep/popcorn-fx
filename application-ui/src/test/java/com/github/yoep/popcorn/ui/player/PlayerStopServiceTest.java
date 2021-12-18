package com.github.yoep.popcorn.ui.player;

import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import com.github.yoep.popcorn.ui.events.PlayMediaEvent;
import com.github.yoep.popcorn.ui.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

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
    private ApplicationEventPublisher eventPublisher;
    @InjectMocks
    private PlayerStopService service;

    @Test
    void testInit_whenInvoked_shouldSubscribeToThePlayerEvents() {
        service.init();

        verify(playerEventService).addListener(isA(PlayerListener.class));
    }

    @Test
    void testInit_whenPlayerIsStoppedAndDurationIsLargerThanZero_shouldPublishClosePlayerEvent() {
        var listenerHolder = new AtomicReference<PlayerListener>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(playerEventService).addListener(isA(PlayerListener.class));

        service.init();
        var playerListener = listenerHolder.get();
        playerListener.onDurationChanged(1000L);
        playerListener.onStateChanged(PlayerState.STOPPED);

        verify(eventPublisher).publishEvent(isA(ClosePlayerEvent.class));
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
        doNothing().when(eventPublisher).publishEvent(isA(ClosePlayerEvent.class));

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
        doNothing().when(eventPublisher).publishEvent(isA(ClosePlayerEvent.class));

        service.init();
        service.onPlayMedia(event);
        var playerListener = listenerHolder.get();
        playerListener.onDurationChanged(500L);
        playerListener.onStateChanged(PlayerState.STOPPED);

        verify(eventPublisher).publishEvent(isA(PlayerStoppedEvent.class));
        var result = eventHolder.get();
        assertEquals(media, result.getMedia().get());
        assertEquals(url, result.getUrl());
        assertEquals(quality, result.getQuality().get());
    }
}

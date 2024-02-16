package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.Handle;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerHeaderServiceTest {
    @Mock
    private VideoService videoService;
    @Mock
    private TorrentService torrentService;
    @Mock
    private PlayerHeaderListener listener;
    @InjectMocks
    private PlayerHeaderService service;

    private final AtomicReference<PlaybackListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));

        service.addListener(listener);
    }

    @Test
    void testPlaybackListener_whenPlayRequestInvoked_shouldSetTheTitle() {
        var expectedTitle = "lorem ipsum dolor";
        var request = mock(PlayRequest.class);
        when(request.getTitle()).thenReturn(expectedTitle);
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onTitleChanged(expectedTitle);
    }

    @Test
    void testPlaybackListener_whenRequestIsMediaPlayRequest_shouldSetTheQuality() {
        var expectedQuality = "1080p";
        var request = mock(PlayRequest.class);
        when(request.getQuality()).thenReturn(Optional.of(expectedQuality));
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onQualityChanged(expectedQuality);
    }

    @Test
    void testPlaybackListener_whenRequestIsStreamingRequest_shouldSetStreamStateToTrue() {
        var request = mock(PlayRequest.class);
        when(request.getStreamHandle()).thenReturn(Optional.of(new Handle(111L)));
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onStreamStateChanged(true);
    }

    @Test
    void testPlaybackListener_whenRequestIsStreamingRequest_shouldInvokeDownloadStatusChangedOnListeners() {
        var listenerHolder = new AtomicReference<TorrentStreamListener>();
        var playbackHolder = new AtomicReference<PlaybackListener>();
        var progress = mock(DownloadStatus.class);
        var request = mock(PlayRequest.class);
        var streamHandle = new Handle(123L);
        when(request.getStreamHandle()).thenReturn(Optional.of(streamHandle));
        when(torrentService.addListener(isA(Handle.class), isA(TorrentStreamListener.class))).thenAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(1, TorrentStreamListener.class));
            return new Handle(222L);
        });
        doAnswer(invocation -> {
            playbackHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));
        service.init();

        playbackHolder.get().onPlay(request);
        listenerHolder.get().onDownloadStatus(progress);

        verify(listener).onDownloadStatusChanged(progress);
    }
}
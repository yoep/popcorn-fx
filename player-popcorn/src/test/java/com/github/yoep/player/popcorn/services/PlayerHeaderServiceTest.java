package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.Handle;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
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

    private PlayerHeaderService service;

    private final AtomicReference<PlaybackListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));

        service = new PlayerHeaderService(videoService, torrentService);
        service.addListener(listener);
    }

    @Test
    void testPlaybackListener_whenPlayRequestInvoked_shouldSetTheTitle() {
        var expectedTitle = "lorem ipsum dolor";
        var request = mock(PlayRequest.class);
        when(request.getTitle()).thenReturn(expectedTitle);

        listenerHolder.get().onPlay(request);

        verify(listener).onTitleChanged(expectedTitle);
    }

    @Test
    void testPlaybackListener_whenRequestIsMediaPlayRequest_shouldSetTheQuality() {
        var expectedQuality = "1080p";
        var request = mock(PlayRequest.class);
        when(request.getQuality()).thenReturn(Optional.of(expectedQuality));

        listenerHolder.get().onPlay(request);

        verify(listener).onQualityChanged(expectedQuality);
    }

    @Test
    void testPlaybackListener_whenRequestIsStreamingRequest_shouldSetStreamStateToTrue() {
        var request = mock(PlayRequest.class);
        when(request.getStreamHandle()).thenReturn(Optional.of(new Handle(111L)));

        listenerHolder.get().onPlay(request);

        verify(listener).onStreamStateChanged(true);
    }

    @Test
    void testPlaybackListener_whenRequestIsStreamingRequest_shouldInvokeDownloadStatusChangedOnListeners() {
        var streamListener = new AtomicReference<TorrentListener>();
        var progress = mock(DownloadStatus.class);
        var request = mock(PlayRequest.class);
        var streamHandle = new Handle(123L);
        when(request.getStreamHandle()).thenReturn(Optional.of(streamHandle));
        doAnswer(invocation -> {
            streamListener.set(invocation.getArgument(1, TorrentListener.class));
            return null;
        }).when(torrentService).addListener(isA(Handle.class), isA(TorrentListener.class));

        listenerHolder.get().onPlay(request);
        streamListener.get().onDownloadStatus(progress);

        verify(listener).onDownloadStatusChanged(progress);
    }
}
package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.player.model.MediaPlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import com.github.yoep.popcorn.backend.player.model.StreamPlayRequest;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerHeaderServiceTest {
    @Mock
    private PopcornPlayer player;
    @Mock
    private VideoService videoService;
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
    void testStop_whenInvokeD_shouldStopThePlayer() {
        service.stop();

        verify(player).stop();
    }

    @Test
    void testPlaybackListener_whenPlayRequestInvoked_shouldSetTheTitle() {
        var expectedTitle = "lorem ipsum dolor";
        var request = SimplePlayRequest.builder()
                .title(expectedTitle)
                .build();
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onTitleChanged(expectedTitle);
    }

    @Test
    void testPlaybackListener_whenRequestIsMediaPlayRequest_shouldSetTheQuality() {
        var expectedQuality = "1080p";
        var torrentStream = mock(TorrentStream.class);
        var request = MediaPlayRequest.mediaBuilder()
                .quality(expectedQuality)
                .torrentStream(torrentStream)
                .build();
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onQualityChanged(expectedQuality);
    }

    @Test
    void testPlaybackListener_whenRequestIsStreamingRequest_shouldSetStreamStateToTrue() {
        var torrentStream = mock(TorrentStream.class);
        var request = StreamPlayRequest.streamBuilder()
                .torrentStream(torrentStream)
                .build();
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onStreamStateChanged(true);
    }

    @Test
    void testPlaybackListener_whenRequestIsStreamingRequest_shouldInvokeDownloadStatusChangedOnListeners() {
        var torrentListenerHolder = new AtomicReference<TorrentListener>();
        var torrentStream = mock(TorrentStream.class);
        var progress = mock(DownloadStatus.class);
        var request = StreamPlayRequest.streamBuilder()
                .torrentStream(torrentStream)
                .build();
        doAnswer(invocation -> {
            torrentListenerHolder.set(invocation.getArgument(0, TorrentListener.class));
            return null;
        }).when(torrentStream).addListener(isA(TorrentListener.class));
        service.init();

        listenerHolder.get().onPlay(request);
        torrentListenerHolder.get().onDownloadStatus(progress);

        verify(listener).onDownloadStatusChanged(progress);
    }
}
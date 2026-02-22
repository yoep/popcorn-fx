package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ServerStream;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Stream;
import com.github.yoep.popcorn.backend.stream.StreamListener;
import com.github.yoep.popcorn.backend.stream.StreamServer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerHeaderServiceTest {
    @Mock
    private VideoService videoService;
    @Mock
    private StreamServer streamServer;
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

        service = new PlayerHeaderService(videoService, streamServer);
        service.addListener(listener);
    }

    @Test
    void testPlaybackListener_whenPlayRequestInvoked_shouldSetTheTitle() {
        var expectedTitle = "lorem ipsum dolor";
        var request = Player.PlayRequest.newBuilder()
                .setTitle(expectedTitle)
                .build();

        listenerHolder.get().onPlay(request);

        verify(listener).onTitleChanged(expectedTitle);
    }

    @Test
    void testPlaybackListener_whenRequestIsMediaPlayRequest_shouldSetTheQuality() {
        var expectedQuality = "1080p";
        var request = Player.PlayRequest.newBuilder()
                .setQuality(expectedQuality)
                .build();

        listenerHolder.get().onPlay(request);

        verify(listener).onQualityChanged(expectedQuality);
    }

    @Test
    void testPlaybackListener_whenRequestIsStreamingRequest_shouldSetStreamStateToTrue() {
        var request = Player.PlayRequest.newBuilder()
                .setStream(ServerStream.newBuilder()
                        .setFilename("test.mp4")
                        .build())
                .build();

        listenerHolder.get().onPlay(request);

        verify(listener).onStreamStateChanged(true);
    }

    @Test
    void testPlaybackListener_whenRequestIsStreamingRequest_shouldInvokeDownloadStatusChangedOnListeners() {
        var streamListener = new AtomicReference<StreamListener>();
        var request = Player.PlayRequest.newBuilder()
                .setStream(ServerStream.newBuilder()
                        .setFilename("test.mp4")
                        .build())
                .build();
        doAnswer(invocation -> {
            streamListener.set(invocation.getArgument(1, StreamListener.class));
            return null;
        }).when(streamServer).addListener(isA(String.class), isA(StreamListener.class));

        listenerHolder.get().onPlay(request);
        streamListener.get().onStatsChanged(Stream.StreamStats.newBuilder()
                .setProgress(10.5f)
                .build());

        verify(listener).onDownloadStatusChanged(isA(DownloadStatus.class));
    }
}
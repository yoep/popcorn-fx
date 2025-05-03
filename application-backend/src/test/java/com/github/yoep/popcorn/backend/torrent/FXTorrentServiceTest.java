package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.CleanTorrentsDirectoryRequest;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Handle;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Torrent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.TorrentEvent;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class FXTorrentServiceTest {
    @Mock
    private FxChannel fxChannel;

    private final AtomicReference<FxCallback<TorrentEvent>> subscriptionHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            subscriptionHolder.set(invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(isA(String.class), isA(Parser.class), isA(FxCallback.class));
    }

    @Test
    void testAddListener() {
        var handle = Handle.newBuilder().setHandle(1).build();
        var event = TorrentEvent.newBuilder()
                .setEvent(TorrentEvent.Event.STATS)
                .setTorrentHandle(handle)
                .setStats(TorrentEvent.Stats.newBuilder()
                        .setStats(Torrent.Stats.newBuilder()
                                .setProgress(0.8f)
                                .build())
                        .build())
                .build();
        var listener = mock(TorrentListener.class);
        var service = new FXTorrentService(fxChannel);

        service.addListener(handle, listener);
        subscriptionHolder.get().callback(event);

        verify(listener).onDownloadStatus(isA(DownloadStatus.class));
    }

    @Test
    void testListener_whenDifferentHandle_shouldNotInvokeListener() {
        var event = TorrentEvent.newBuilder()
                .setEvent(TorrentEvent.Event.STATS)
                .setTorrentHandle(Handle.newBuilder()
                        .setHandle(11)
                        .build())
                .setStats(TorrentEvent.Stats.newBuilder()
                        .setStats(Torrent.Stats.newBuilder()
                                .setProgress(0.8f)
                                .build())
                        .build())
                .build();
        var handle = Handle.newBuilder().setHandle(123).build();
        var listener = mock(TorrentListener.class);
        var service = new FXTorrentService(fxChannel);

        service.addListener(handle, listener);
        subscriptionHolder.get().callback(event);

        verify(listener, times(0)).onDownloadStatus(isA(DownloadStatus.class));
    }

    @Test
    void testCleanup() {
        var service = new FXTorrentService(fxChannel);

        service.cleanup();

        verify(fxChannel).send(isA(CleanTorrentsDirectoryRequest.class));
    }
}
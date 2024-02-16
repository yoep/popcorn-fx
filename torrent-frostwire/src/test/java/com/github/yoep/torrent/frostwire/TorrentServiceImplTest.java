package com.github.yoep.torrent.frostwire;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentHealthState;
import com.github.yoep.popcorn.backend.lib.Handle;
import com.github.yoep.popcorn.backend.torrent.DownloadStatusC;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamEventC;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamEventCallback;
import com.github.yoep.torrent.frostwire.model.TorrentHealthImpl;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class TorrentServiceImplTest {
    @Mock
    private TorrentSessionManager sessionManager;
    @Mock
    private TorrentResolverService torrentResolverService;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private TorrentServiceImpl service;

    @Test
    void testCalculateHealth_whenSeedsIsZeroAndPeersIsZero_shouldReturnUnknown() {
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.UNKNOWN, 0, 0, 0);

        var result = service.calculateHealth(0, 0);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCalculateHealth_whenPeersIsLargerThanSeeds_shouldReturnBad() {
        var seeds = 5;
        var peers = 10;
        var ratio = 0.5;
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.BAD, ratio, seeds, peers);

        var result = service.calculateHealth(seeds, peers);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCalculateHealth_whenSeedsIsEqualToPeers_shouldReturnBad() {
        var seeds = 10;
        var peers = 10;
        var ratio = 1;
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.BAD, ratio, seeds, peers);

        var result = service.calculateHealth(seeds, peers);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCalculateHealth_whenSeedsIsLargerThan30_shouldReturnGood() {
        var seeds = 35;
        var peers = 10;
        var ratio = 3.5;
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.GOOD, ratio, seeds, peers);

        var result = service.calculateHealth(seeds, peers);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCalculateHealth_whenRatioIsLargerThan5_shouldReturnExcellent() {
        var seeds = 50;
        var peers = 10;
        var ratio = 5;
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.EXCELLENT, ratio, seeds, peers);

        var result = service.calculateHealth(seeds, peers);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCleanup() {
        service.cleanup();

        verify(fxLib).cleanup_torrents_directory(instance);
    }

    @Test
    void testAddStreamListener() {
        var callbackHandle = 8566L;
        var handle = new Handle(123L);
        var callbackHolder = new AtomicReference<TorrentStreamEventCallback>();
        var listener = mock(TorrentStreamListener.class);
        var event = new TorrentStreamEventC.ByValue();
        var downloadStatus = DownloadStatusC.ByValue.builder()
                .progress(0.8f)
                .seeds(2)
                .peers(10)
                .downloaded(13L)
                .build();
        event.tag = TorrentStreamEventC.Tag.DOWNLOAD_STATUS;
        event.union = new TorrentStreamEventC.TorrentStreamEventCUnion();
        event.union.downloadStatus_body = new TorrentStreamEventC.DownloadStatus_Body();
        event.union.downloadStatus_body.status = downloadStatus;
        when(fxLib.register_torrent_stream_event_callback(isA(PopcornFx.class), isA(Long.class), isA(TorrentStreamEventCallback.class))).thenAnswer(invocation -> {
            callbackHolder.set(invocation.getArgument(2, TorrentStreamEventCallback.class));
            return callbackHandle;
        });

        var result = service.addListener(handle, listener);
        assertEquals(callbackHandle, result.nativeHandle());
        verify(fxLib).register_torrent_stream_event_callback(eq(instance), eq(handle.nativeHandle()), isA(TorrentStreamEventCallback.class));

        var callback = callbackHolder.get();
        callback.callback(event);
        verify(listener).onDownloadStatus(downloadStatus);
    }

    @Test
    void testRemoveStreamListener() {
        var callbackHandleValue = 84555222L;
        var streamHandleValue = 666L;
        var handle = new Handle(streamHandleValue);
        var listener = mock(TorrentStreamListener.class);
        when(fxLib.register_torrent_stream_event_callback(isA(PopcornFx.class), isA(Long.class), isA(TorrentStreamEventCallback.class))).thenReturn(callbackHandleValue);

        var callbackHandle = service.addListener(handle, listener);
        service.removeListener(callbackHandle);

        verify(fxLib).remove_torrent_stream_event_callback(instance, streamHandleValue, callbackHandleValue);
    }
}

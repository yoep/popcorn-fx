package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentListener;
import com.github.yoep.popcorn.backend.lib.Handle;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class DefaultTorrentServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private DefaultTorrentService service;

    @BeforeEach
    void setUp() {
        FxLib.INSTANCE.set(fxLib);
    }

    @Test
    void testAddListener() {
        var listener = mock(TorrentListener.class);
        var callbackHolder = new AtomicReference<TorrentEventCallback>();
        var downloadStatus = new DownloadStatusC.ByValue();
        var event = new TorrentEventC.ByValue();
        event.tag = TorrentEventC.Tag.DOWNLOAD_STATUS;
        event.union = new TorrentEventC.TorrentEventCUnion();
        event.union.downloadStatus_body = new TorrentEventC.DownloadStatus_Body();
        event.union.downloadStatus_body.status = downloadStatus;
        doAnswer(invocation -> {
            callbackHolder.set(invocation.getArgument(2, TorrentEventCallback.class));
            return null;
        }).when(fxLib).register_torrent_event_callback(eq(instance), isA(Long.class), isA(TorrentEventCallback.class));

        service.addListener(new Handle(13L), listener);

        var callback = callbackHolder.get();
        assertNotNull(callback);
        callback.callback(event);

        verify(listener).onDownloadStatus(downloadStatus);
    }
}
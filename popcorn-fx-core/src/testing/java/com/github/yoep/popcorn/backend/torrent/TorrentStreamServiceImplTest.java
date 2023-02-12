package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@Disabled
@ExtendWith(MockitoExtension.class)
class TorrentStreamServiceImplTest {
    @Mock
    private FxLib lib;
    @Mock
    private PopcornFxInstance instance;
    @Mock
    private PopcornFx fxInstance;
    @Mock
    private TorrentService torrentService;
    @InjectMocks
    private TorrentStreamServiceImpl service;

    @Test
    public void testStopStream() {
        var torrent = mock(Torrent.class);
        var pointer = mock(TorrentWrapperPointer.class);
        var streamWrapper = mock(TorrentStreamWrapper.class);
        when(instance.get()).thenReturn(fxInstance);
        when(lib.torrent_wrapper(isA(TorrentWrapper.ByValue.class))).thenReturn(pointer);
        when(lib.start_stream(isA(PopcornFx.class), isA(TorrentWrapperPointer.class))).thenReturn(streamWrapper);

        var stream = service.startStream(torrent);
        service.stopStream(stream);

        verify(lib).start_stream(fxInstance, pointer);
        verify(lib).stop_stream(fxInstance, streamWrapper);
    }
}
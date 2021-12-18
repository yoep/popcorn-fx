package com.github.yoep.torrent.stream.services;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.torrent.stream.models.TorrentStreamImpl;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.boot.autoconfigure.web.ServerProperties;

import java.io.File;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class TorrentStreamServiceImplTest {
    private static final String PORT = "9999";

    @Mock
    private TorrentService torrentService;
    @Mock
    private ServerProperties serverProperties;

    private TorrentStreamServiceImpl torrentStreamService;

    @BeforeEach
    void setUp() {
        torrentStreamService = new TorrentStreamServiceImpl(torrentService, serverProperties);
    }

    @Test
    void testStartStream_whenTorrentIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> torrentStreamService.startStream(null), "torrent cannot be null");
    }

    @Test
    void testStartStream_whenInvoked_shouldRetrnTheTorrentStreamForTheGivenTorrent() {
        var torrent = mock(Torrent.class);
        var filename = "my-video.mp4";
        var url = "http://127.0.0.1:9999/video/" + filename;
        var expectedResult = new TorrentStreamImpl(torrent, url);
        var file = mock(File.class);
        when(torrent.getFile()).thenReturn(file);
        when(file.getAbsolutePath()).thenReturn("/" + filename);

        var result = torrentStreamService.startStream(torrent);

        assertEquals(expectedResult, result);
    }

    @Test
    void testStopStream_whenTorrentStreamIsUnknown_shouldDoNothing() {
        var torrentStream = mock(TorrentStream.class);
        var file = mock(File.class);
        var filename = "lorem.mp4";
        when(torrentStream.getFile()).thenReturn(file);
        when(file.getAbsolutePath()).thenReturn(filename);

        torrentStreamService.stopStream(torrentStream);

        verify(torrentService, times(0)).remove(isA(Torrent.class));
    }

    @Test
    void testStopStream_whenTorrentStreamIsKnown_shouldRemoveTheTorrentFromTheTorrentService() {
        var torrent = mock(Torrent.class);
        var file = mock(File.class);
        var filename = "lorem.mp4";
        when(torrent.getFile()).thenReturn(file);
        when(file.getAbsolutePath()).thenReturn(filename);

        var torrentStream = torrentStreamService.startStream(torrent);
        torrentStreamService.stopStream(torrentStream);

        verify(torrentService).remove(torrent);
    }
}

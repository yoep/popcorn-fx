package com.github.yoep.torrent.stream.models;

import com.github.yoep.popcorn.backend.adapters.torrent.FailedToPrepareTorrentStreamException;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;


class TorrentStreamImplTest {
    @Test
    void testConstructor_whenPreparePiecesCouldNotBeDetermined_shouldThrowFailedToPrepareTorrentStreamException() {
        var torrent = mock(Torrent.class);
        when(torrent.getTotalPieces()).thenReturn(0);

        assertThrows(FailedToPrepareTorrentStreamException.class, () -> new TorrentStreamImpl(torrent, "http://my-stream-url"));
    }
}
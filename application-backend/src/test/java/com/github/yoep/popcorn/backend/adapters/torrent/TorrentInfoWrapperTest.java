package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class TorrentInfoWrapperTest {
    @Test
    void testNewInstanceFromInterface() {
        var fileInfo = mock(TorrentFileInfo.class);
        var info = mock(com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo.class);
        var magnetUri = "magnet:?MyFooMagnetUri";
        var name = "FooBarName";
        when(info.getMagnetUri()).thenReturn(magnetUri);
        when(info.getName()).thenReturn(name);
        when(info.getFiles()).thenReturn(Collections.singletonList(fileInfo));
        when(info.getTotalFiles()).thenReturn(1);

        var result = new TorrentInfoWrapper(info);

        assertEquals(magnetUri, result.magnetUri);
        assertEquals(name, result.name);
        assertEquals(1, result.totalFiles);
    }
}
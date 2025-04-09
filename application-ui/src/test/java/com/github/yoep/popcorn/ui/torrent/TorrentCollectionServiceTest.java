package com.github.yoep.popcorn.ui.torrent;

import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class TorrentCollectionServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private TorrentCollectionService service;

    @Test
    void testIsStored() {
        var magnetUri = "magnet:MyMagnetUri";
        when(fxLib.torrent_collection_is_stored(isA(PopcornFx.class), isA(String.class))).thenReturn((byte) 1);

        var result = service.isStored(magnetUri);

        assertTrue(result);
        verify(fxLib).torrent_collection_is_stored(instance, magnetUri);
    }

    @Test
    void testAddTorrent() {
        var torrent = mock(TorrentInfo.class);
        var magnetUri = "magnet:UriToStore";
        var name = "Foo";
        when(torrent.getName()).thenReturn(name);
        when(torrent.getMagnetUri()).thenReturn(magnetUri);

        service.addTorrent(torrent);

        verify(fxLib).torrent_collection_add(instance, name, magnetUri);
    }
}
package com.github.yoep.popcorn.ui.torrent;

import com.github.yoep.popcorn.backend.storage.StorageService;
import com.github.yoep.popcorn.ui.torrent.models.StoredTorrent;
import com.github.yoep.popcorn.ui.torrent.models.TorrentCollection;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class TorrentCollectionServiceTest {
    @Mock
    private StorageService storageService;
    @InjectMocks
    private TorrentCollectionService service;

    @Test
    void testIsStored_whenMagnetIsStored_shouldReturnTrue() {
        var uri = "my-uri";
        var collection = TorrentCollection.builder()
                .torrents(Collections.singletonList(StoredTorrent.builder()
                        .magnetUri(uri)
                        .build()))
                .build();
        when(storageService.read(TorrentCollectionService.STORAGE_NAME, TorrentCollection.class)).thenReturn(Optional.of(collection));

        var result = service.isStored(uri);

        assertTrue(result);
    }

    @Test
    void testGetStoredTorrent_whenInvoked_shouldReturnStoredTorrents() {
        var storedTorrents = Collections.singletonList(StoredTorrent.builder()
                .name("ipsum")
                .magnetUri("lorem")
                .build());
        var collection = TorrentCollection.builder()
                .torrents(storedTorrents)
                .build();
        when(storageService.read(TorrentCollectionService.STORAGE_NAME, TorrentCollection.class))
                .thenReturn(Optional.of(collection));

        var result = service.getStoredTorrents();

        assertEquals(storedTorrents, result);
    }
}
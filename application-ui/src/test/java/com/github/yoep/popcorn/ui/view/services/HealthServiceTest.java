package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.torrent.adapter.TorrentService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class HealthServiceTest {
    @Mock
    private TorrentService torrentService;
    @InjectMocks
    private HealthService healthService;

    @Test
    void testCalculateHealth_whenInvoked_shouldCallCalculateHealthOnTorrentService() {
        var seeds = 10;
        var peers = 20;

        healthService.calculateHealth(seeds, peers);

        verify(torrentService).calculateHealth(seeds, peers);
    }

    @Test
    void testGetTorrentHealth_whenPreviousFutureIsStillRunning_shouldCancelPreviousFuture() {
        var firstUrl = "lorem";
        var secondUrl = "ipsum";
        var future = mock(CompletableFuture.class);
        when(torrentService.getTorrentHealth(firstUrl)).thenReturn(future);
        when(future.isDone()).thenReturn(false);

        healthService.getTorrentHealth(firstUrl);
        healthService.getTorrentHealth(secondUrl);

        verify(future).cancel(true);
    }

    @Test
    void testOnLoadMediaTorrent_whenPreviousFutureIsStillRunning_shouldCancelPreviousFuture() {
        var firstUrl = "lorem";
        var future = mock(CompletableFuture.class);
        when(torrentService.getTorrentHealth(firstUrl)).thenReturn(future);
        when(future.isDone()).thenReturn(false);

        healthService.getTorrentHealth(firstUrl);
        healthService.onCancelHealthRetrieval();

        verify(future).cancel(true);
    }
}

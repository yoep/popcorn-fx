package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.TorrentSettings;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.File;
import java.util.concurrent.CompletableFuture;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class HealthServiceTest {
    @Mock
    private TorrentService torrentService;
    @Mock
    private ApplicationConfig settingsService;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private TorrentSettings torrentSettings;
    @Mock
    private File torrentDirectory;
    @InjectMocks
    private HealthService healthService;

    @BeforeEach
    void setUp() {
        lenient().when(settingsService.getSettings()).thenReturn(settings);
        lenient().when(settings.getTorrentSettings()).thenReturn(torrentSettings);
        lenient().when(torrentSettings.getDirectory()).thenReturn(torrentDirectory);
    }

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
        when(torrentService.getTorrentHealth(firstUrl, torrentDirectory)).thenReturn(future);
        when(torrentService.getTorrentHealth(secondUrl, torrentDirectory)).thenReturn(future);
        when(future.isDone()).thenReturn(false);

        healthService.getTorrentHealth(firstUrl);
        healthService.getTorrentHealth(secondUrl);

        verify(future).cancel(true);
    }

    @Test
    void testOnLoadMediaTorrent_whenPreviousFutureIsStillRunning_shouldCancelPreviousFuture() {
        var firstUrl = "lorem";
        var future = mock(CompletableFuture.class);
        when(torrentService.getTorrentHealth(firstUrl, torrentDirectory)).thenReturn(future);
        when(future.isDone()).thenReturn(false);

        healthService.getTorrentHealth(firstUrl);
        healthService.onCancelHealthRetrieval();

        verify(future).cancel(true);
    }
}

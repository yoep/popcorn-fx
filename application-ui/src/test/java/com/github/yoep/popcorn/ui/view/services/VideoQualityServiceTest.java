package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Arrays;
import java.util.HashMap;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class VideoQualityServiceTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @InjectMocks
    private VideoQualityService service;

    @Test
    void testGetDefaultVideoResolution() {
        var torrents = new HashMap<String, Media.TorrentInfo>() {{
            put("480p", Media.TorrentInfo.getDefaultInstance());
            put("720p", Media.TorrentInfo.getDefaultInstance());
            put("3Dp", Media.TorrentInfo.getDefaultInstance());
            put("1080p", Media.TorrentInfo.getDefaultInstance());
        }};
        var expectedResult = new String[]{"480p", "720p", "1080p"};

        var result = service.getVideoResolutions(Media.TorrentQuality.newBuilder()
                .putAllQualities(torrents)
                .build());

        assertArrayEquals(expectedResult, result);
    }

    @Test
    void testGetDefaultResolution() {
        var resolutions = Arrays.asList("480p", "720p", "1080p");
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setPlaybackSettings(ApplicationSettings.PlaybackSettings.newBuilder()
                        .setQuality(ApplicationSettings.PlaybackSettings.Quality.P720)
                        .build())
                .build()));

        var result = service.getDefaultVideoResolution(resolutions).resultNow();

        assertEquals("720p", result);
    }
}
package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.PlaybackSettings;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Arrays;
import java.util.HashMap;
import java.util.Optional;

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
        var torrents = new HashMap<String, MediaTorrentInfo>() {{
            put("480p", mock(MediaTorrentInfo.class));
            put("720p", mock(MediaTorrentInfo.class));
            put("3Dp", mock(MediaTorrentInfo.class));
            put("1080p", mock(MediaTorrentInfo.class));
        }};
        var expectedResult = new String[]{"480p", "720p", "1080p"};

        var result = service.getVideoResolutions(torrents);

        assertArrayEquals(expectedResult, result);
    }

    @Test
    void testGetDefaultResolution() {
        var applicationSettings = mock(ApplicationSettings.class);
        var playbackSettings = mock(PlaybackSettings.class);
        var resolutions = Arrays.asList("480p", "720p", "1080p");
        when(applicationConfig.getSettings()).thenReturn(applicationSettings);
        when(applicationSettings.getPlaybackSettings()).thenReturn(playbackSettings);
        when(playbackSettings.getQuality()).thenReturn(Optional.of(PlaybackSettings.Quality.p720));

        var result = service.getDefaultVideoResolution(resolutions);

        assertEquals("720p", result);
    }
}
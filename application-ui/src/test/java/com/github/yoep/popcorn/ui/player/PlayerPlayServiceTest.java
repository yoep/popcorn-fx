package com.github.yoep.popcorn.ui.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.PlaybackSettings;
import com.github.yoep.popcorn.ui.media.resume.AutoResumeService;
import com.github.yoep.popcorn.ui.player.model.SimplePlayRequest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerPlayServiceTest {
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private AutoResumeService autoResumeService;
    @Mock
    private ScreenService screenService;
    @Mock
    private SettingsService settingsService;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private PlaybackSettings playbackSettings;
    @InjectMocks
    private PlayerPlayService service;

    @Test
    void testOnPlayVideo_whenThereIsNoActivePlayer_shouldNotThrowAnException() {
        var event = mock(PlayVideoEvent.class);
        when(event.getUrl()).thenReturn("my play video event url");
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.empty());

        service.onPlayVideo(event);

        assertTrue(true);
    }

    @Test
    void testOnPlayVideo_whenPlayerIsActive_shouldInvokedPlayOnThePlayer() {
        var url = "my video url";
        var title = "my video title";
        var event = new PlayVideoEvent(this, url, title, true);
        var player = mock(Player.class);
        var expectedResult = SimplePlayRequest.builder()
                .url(url)
                .title(title)
                .build();
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(settingsService.getSettings()).thenReturn(settings);
        when(settings.getPlaybackSettings()).thenReturn(playbackSettings);

        service.onPlayVideo(event);

        verify(player).play(expectedResult);
    }

    @Test
    void testOnPlayVideo_whenFullscreenPlaybackIsEnabled_shouldActiveFullscreenMode() {
        var event = mock(PlayVideoEvent.class);
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(settingsService.getSettings()).thenReturn(settings);
        when(settings.getPlaybackSettings()).thenReturn(playbackSettings);
        when(playbackSettings.isFullscreen()).thenReturn(true);

        service.onPlayVideo(event);

        verify(screenService).fullscreen(true);
    }

    @Test
    void testOnPlayerVideo_whenEventIsPlayerMediaEvent_shouldAutoResumeTheLastKnownMediaTimestamp() {
        var mediaId = "t12356";
        var filename = "my-filename.mp4";
        var expectedTimestamp = 845669L;
        var media = mock(Media.class);
        var event = mock(PlayMediaEvent.class);
        var player = mock(Player.class);
        when(event.getMedia()).thenReturn(media);
        when(event.getUrl()).thenReturn(filename);
        when(media.getId()).thenReturn(mediaId);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(autoResumeService.getResumeTimestamp(isA(String.class), isA(String.class))).thenReturn(Optional.of(expectedTimestamp));
        when(settingsService.getSettings()).thenReturn(settings);
        when(settings.getPlaybackSettings()).thenReturn(playbackSettings);

        service.onPlayVideo(event);

        verify(autoResumeService).getResumeTimestamp(mediaId, filename);
        verify(player).seek(expectedTimestamp);
    }
}

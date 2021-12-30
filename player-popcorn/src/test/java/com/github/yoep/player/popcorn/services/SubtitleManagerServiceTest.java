package com.github.yoep.player.popcorn.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayer;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleMatcher;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import java.io.File;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class SubtitleManagerServiceTest {
    @Mock
    private SettingsService settingsService;
    @Mock
    private VideoService videoService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private SubtitlePickerService subtitlePickerService;
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private ApplicationSettings settings;
    @InjectMocks
    private SubtitleManagerService service;

    private final ObjectProperty<Subtitle> activeSubtitleProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        lenient().when(settingsService.getSettings()).thenReturn(settings);
        lenient().when(subtitleService.activeSubtitleProperty()).thenReturn(activeSubtitleProperty);
    }

    @Test
    void testUpdateSubtitleOffset_whenOffsetIsGiven_shouldUpdateTheOffsetValue() {
        var value = 100;

        service.updateSubtitleOffset(value);
        var result = service.getSubtitleOffset();

        assertEquals(value, result);
    }

    @Test
    void testUpdateSubtitleOffset_whenVideoIsPresentAndSupportNativeSubtitles_shouldUpdateTheOffsetInTheVideo() {
        var value = 800;
        var videoPlayer = mock(VideoPlayer.class);
        when(videoService.getVideoPlayer()).thenReturn(Optional.of(videoPlayer));
        when(videoPlayer.supportsNativeSubtitleFile()).thenReturn(true);

        service.updateSubtitleOffset(value);

        verify(videoPlayer).subtitleDelay(value);
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsNone_shouldDisableSubtitleTrack() {
        service.updateSubtitle(SubtitleInfo.none());

        verify(subtitleService).setActiveSubtitle(Subtitle.none());
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsGivenAndNotCustom_shouldDownloadAndActivateTheSubtitle() {
        var subtitleInfo = mock(SubtitleInfo.class);
        var subtitle = mock(Subtitle.class);
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.class))).thenReturn(CompletableFuture.completedFuture(subtitle));

        service.updateSubtitle(subtitleInfo);

        verify(subtitleService).setActiveSubtitle(subtitle);
    }

    @Test
    void testUpdateSubtitle_whenSubtitleDownloadFails_shouldPublishErrorNotification() {
        var expectedErrorText = "my error text";
        var subtitleInfo = mock(SubtitleInfo.class);
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.class)))
                .thenReturn(CompletableFuture.failedFuture(new RuntimeException("my subtitle exception")));
        when(localeText.get(VideoMessage.SUBTITLE_DOWNLOAD_FILED)).thenReturn(expectedErrorText);

        service.updateSubtitle(subtitleInfo);

        verify(eventPublisher).publishEvent(new ErrorNotificationEvent(service, expectedErrorText));
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsDownloadedAndVideoPlayerSupportNativeSubtitle_shouldUpdateSubtitleWithinVideoPlayer() {
        var subtitleInfo = mock(SubtitleInfo.class);
        var subtitle = mock(Subtitle.class);
        var videoPlayer = mock(VideoPlayer.class);
        var subtitleFile = new File(".");
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.class))).thenReturn(CompletableFuture.completedFuture(subtitle));
        when(videoService.getVideoPlayer()).thenReturn(Optional.of(videoPlayer));
        when(videoPlayer.supportsNativeSubtitleFile()).thenReturn(true);
        when(subtitle.getFile()).thenReturn(subtitleFile);

        service.updateSubtitle(subtitleInfo);

        verify(videoPlayer).subtitleFile(subtitleFile);
    }

    @Test
    void testSubtitleSettingsListener_whenSubtitleSizeIsChangeD_shouldUpdateSubtitleSize() {
        var expectedValue = 34;
        var subtitleSettings = SubtitleSettings.builder()
                .fontSize(12)
                .build();
        when(settings.getSubtitleSettings()).thenReturn(subtitleSettings);
        service.init();

        subtitleSettings.setFontSize(expectedValue);
        var result = service.getSubtitleSize();

        assertEquals(expectedValue, result);
    }
}
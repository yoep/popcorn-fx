package com.github.yoep.player.popcorn.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.File;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class SubtitleManagerServiceTest {
    @Mock
    private ApplicationConfig settingsService;
    @Mock
    private VideoService videoService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private SubtitlePickerService subtitlePickerService;
    @Mock
    private LocaleText localeText;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ApplicationSettings settings;
    @Mock
    private SubtitleSettings.ByValue subtitleSettings;
    @Mock
    private SubtitleInfo subtitleNone;
    @InjectMocks
    private SubtitleManagerService service;

    private final ObjectProperty<Subtitle> activeSubtitleProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        lenient().when(settingsService.getSettings()).thenReturn(settings);
        lenient().when(settings.getSubtitleSettings()).thenReturn(subtitleSettings);
        lenient().when(subtitleService.activeSubtitleProperty()).thenReturn(activeSubtitleProperty);
        lenient().when(subtitleNone.isNone()).thenReturn(true);
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
        var videoPlayer = mock(VideoPlayback.class);
        when(videoService.getVideoPlayer()).thenReturn(Optional.of(videoPlayer));
        when(videoPlayer.supportsNativeSubtitleFile()).thenReturn(true);

        service.updateSubtitleOffset(value);

        verify(videoPlayer).subtitleDelay(value);
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsNone_shouldDisableSubtitleTrack() {
        service.updateSubtitle(subtitleNone);

        verify(subtitleService).disableSubtitle();
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsNull_shouldDisableSubtitleTrack() {
        service.updateSubtitle(null);

        verify(subtitleService).disableSubtitle();
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsGivenAndNotCustom_shouldDownloadAndActivateTheSubtitle() {
        var subtitleInfo = mock(SubtitleInfo.class);
        var subtitle = mock(Subtitle.class);
        when(subtitleInfo.getLanguage()).thenReturn(SubtitleLanguage.DUTCH);
        when(subtitleService.preferredSubtitle()).thenReturn(Optional.of(mock(SubtitleInfo.class)));
        when(subtitleService.downloadAndParse(isA(SubtitleInfo.class), isA(SubtitleMatcher.class))).thenReturn(CompletableFuture.completedFuture(subtitle));

        service.updateSubtitle(subtitleInfo);

        verify(subtitleService).downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.class));
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
        var videoPlayer = mock(VideoPlayback.class);
        var subtitleFile = new File(".");
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.class))).thenReturn(CompletableFuture.completedFuture(subtitle));
        when(videoService.getVideoPlayer()).thenReturn(Optional.of(videoPlayer));
        when(videoPlayer.supportsNativeSubtitleFile()).thenReturn(true);
        when(subtitle.getFile()).thenReturn(subtitleFile);

        service.updateSubtitle(subtitleInfo);

        verify(videoPlayer).subtitleFile(subtitleFile);
    }

    @Test
    void testSubtitleListener_whenSubtitleIsChangedToCustom_shouldLetTheUserPickASubtitle() {
        var url = "my-video-url";
        var quality = "720p";
        var title = "my-video-title";
        var subtitle = mock(Subtitle.class);
        var custom = mock(SubtitleInfo.class);
        var videoEvent = PlayVideoEvent.builder()
                .source(this)
                .url(url)
                .title(title)
                .build();
        var mediaUrl = PlayMediaEvent.mediaBuilder()
                .source(this)
                .title(title)
                .url(url)
                .quality(quality)
                .media(MovieDetails.builder()
                        .images(Images.builder().build())
                        .build())
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .build();
        var expected_filepath = "/lorem/ipsum.srt";
        when(custom.isCustom()).thenReturn(true);
        when(subtitle.getSubtitleInfo()).thenReturn(Optional.of(custom));
        when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.of(expected_filepath));
        service.init();
        eventPublisher.publish(videoEvent);
        eventPublisher.publish(mediaUrl);

        activeSubtitleProperty.set(subtitle);

        verify(subtitleService).updateCustomSubtitle(expected_filepath);
    }

    @Test
    void testSubtitleListener_whenSubtitleIsChangedToCustomAndUserCancels_shouldDisableTheSubtitleTrack() {
        var subtitle = mock(Subtitle.class);
        var custom = mock(SubtitleInfo.class);
        when(custom.isCustom()).thenReturn(true);
        when(subtitle.getSubtitleInfo()).thenReturn(Optional.of(custom));
        when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.empty());
        service.init();

        activeSubtitleProperty.set(subtitle);

        verify(subtitleService).disableSubtitle();
    }

    @Test
    void testSubtitleSettingsListener_whenSubtitleSizeIsChangeD_shouldUpdateSubtitleSize() {
        var expectedValue = 34;
        var subtitleSettings = mock(SubtitleSettings.ByValue.class);
        when(subtitleSettings.getFontSize()).thenReturn(expectedValue);
        when(settings.getSubtitleSettings()).thenReturn(subtitleSettings);
        service.init();

        subtitleSettings.setFontSize(expectedValue);
        var result = service.getSubtitleSize();

        assertEquals(expectedValue, result);
    }
}
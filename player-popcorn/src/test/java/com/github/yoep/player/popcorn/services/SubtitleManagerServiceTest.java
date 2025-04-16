package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.SubtitleListener;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.services.SubtitlePickerService;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.File;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

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

    private final AtomicReference<SubtitleEventCallback> listenerHolder = new AtomicReference<>();
    private final AtomicReference<PlaybackListener> playbackListenerHolder = new AtomicReference<>();
    private final ObjectProperty<VideoPlayback> videoPlaybackProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        lenient().when(settingsService.getSettings()).thenReturn(settings);
        lenient().when(settings.getSubtitleSettings()).thenReturn(subtitleSettings);
        lenient().when(subtitleNone.isNone()).thenReturn(true);
        lenient().when(videoService.videoPlayerProperty()).thenReturn(videoPlaybackProperty);
        lenient().doAnswer(invocation -> {
            playbackListenerHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0));
            return null;
        }).when(subtitleService).register(isA(com.github.yoep.popcorn.backend.subtitles.model.SubtitleEventCallback.class));
    }

    @Test
    void testUpdateSubtitleOffset_whenOffsetIsGiven_shouldUpdateTheOffsetValue() {
        var value = 100;
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

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
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        service.updateSubtitleOffset(value);

        verify(videoPlayer).subtitleDelay(value);
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsNone_shouldDisableSubtitleTrack() {
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        service.updateSubtitle(subtitleNone);

        verify(subtitleService).disableSubtitle();
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsNull_shouldDisableSubtitleTrack() {
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        service.updateSubtitle(null);

        verify(subtitleService).disableSubtitle();
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsGivenAndNotCustom_shouldDownloadAndActivateTheSubtitle() {
        var language = SubtitleLanguage.DUTCH;
        var subtitleInfo = mock(SubtitleInfo.class);
        var subtitle = mock(Subtitle.class);
        var request = mock(PlayRequest.class);
        when(subtitleInfo.language()).thenReturn(language);
        when(subtitleService.preference()).thenReturn(new SubtitlePreference(SubtitlePreferenceTag.LANGUAGE, language));
        when(subtitleService.downloadAndParse(isA(SubtitleInfo.class), isA(SubtitleMatcher.ByReference.class))).thenReturn(CompletableFuture.completedFuture(subtitle));
        when(request.getUrl()).thenReturn("http://localhost:9000/MyVideo.mp4");
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);
        var listener = playbackListenerHolder.get();
        listener.onPlay(request);

        service.updateSubtitle(subtitleInfo);

        verify(subtitleService).downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.ByReference.class));
    }

    @Test
    void testUpdateSubtitle_whenSubtitleDownloadFails_shouldPublishErrorNotification() {
        var language = SubtitleLanguage.FINNISH;
        var expectedErrorText = "my error text";
        var subtitleInfo = mock(SubtitleInfo.class);
        var request = mock(PlayRequest.class);
        when(request.getUrl()).thenReturn("http://localhost:9000/MyVideo.mp4");
        when(subtitleInfo.language()).thenReturn(language);
        when(subtitleService.preference()).thenReturn(new SubtitlePreference(SubtitlePreferenceTag.LANGUAGE, language));
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.ByReference.class)))
                .thenReturn(CompletableFuture.failedFuture(new RuntimeException("my subtitle exception")));
        when(localeText.get(VideoMessage.SUBTITLE_DOWNLOAD_FILED)).thenReturn(expectedErrorText);
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);
        var listener = playbackListenerHolder.get();
        listener.onPlay(request);

        service.updateSubtitle(subtitleInfo);

        verify(eventPublisher).publishEvent(new ErrorNotificationEvent(service, expectedErrorText));
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsDownloadedAndVideoPlayerSupportNativeSubtitle_shouldUpdateSubtitleWithinVideoPlayer() {
        var subtitleInfo = mock(SubtitleInfo.class);
        var subtitle = mock(Subtitle.class);
        var videoPlayer = mock(VideoPlayback.class);
        var subtitleFile = new File(".");
        var request = mock(PlayRequest.class);
        when(subtitleService.preference()).thenReturn(new SubtitlePreference(SubtitlePreferenceTag.LANGUAGE, SubtitleLanguage.GERMAN));
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.ByReference.class))).thenReturn(CompletableFuture.completedFuture(subtitle));
        when(videoService.getVideoPlayer()).thenReturn(Optional.of(videoPlayer));
        when(videoPlayer.supportsNativeSubtitleFile()).thenReturn(true);
        when(subtitle.getFile()).thenReturn(subtitleFile);
        when(request.getUrl()).thenReturn("http://localhost:9000/MyVideo.mp4");
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);
        var listener = playbackListenerHolder.get();
        listener.onPlay(request);

        service.updateSubtitle(subtitleInfo);

        verify(videoPlayer).subtitleFile(subtitleFile);
    }

    @Test
    void testSubtitleListener_whenSubtitleIsChangedToCustom_shouldLetTheUserPickASubtitle() {
        var subtitleInfo = SubtitleInfo.builder()
                .language(SubtitleLanguage.CUSTOM)
                .files(new SubtitleFile[0])
                .build();
        var event = new SubtitleEvent(SubtitleEventTag.SubtitleInfoChanged, subtitleInfo);
        var request = mock(PlayRequest.class);
        var expected_filepath = "/lorem/ipsum.srt";
        when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.of(expected_filepath));
        when(request.getUrl()).thenReturn("http://localhost:9000/MyVideo.mp4");
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);
        var playbackListener = playbackListenerHolder.get();
        playbackListener.onPlay(request);

        var listener = listenerHolder.get();
        listener.callback(event);

        verify(subtitleService).updatePreferredLanguage(SubtitleLanguage.CUSTOM);
    }

    @Test
    void testSubtitleListener_whenSubtitleIsChangedToCustomAndUserCancels_shouldDisableTheSubtitleTrack() {
        var custom = SubtitleInfo.builder()
                .language(SubtitleLanguage.CUSTOM)
                .files(new SubtitleFile[0])
                .build();
        var event = new SubtitleEvent(SubtitleEventTag.SubtitleInfoChanged, custom);
        when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.empty());
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        var listener = listenerHolder.get();
        listener.callback(event);

        verify(subtitleService).disableSubtitle();
    }

    @Test
    void testSubtitleSettingsListener_whenSubtitleSizeIsChangeD_shouldUpdateSubtitleSize() {
        var expectedValue = 34;
        var subtitleSettings = mock(SubtitleSettings.ByValue.class);
        when(subtitleSettings.getFontSize()).thenReturn(expectedValue);
        when(settings.getSubtitleSettings()).thenReturn(subtitleSettings);
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        subtitleSettings.setFontSize(expectedValue);
        var result = service.getSubtitleSize();

        assertEquals(expectedValue, result);
    }

    @Test
    void testVideoPlayerProperty_shouldUpdateSubtitle() throws ExecutionException, InterruptedException, TimeoutException {
        var subtitleInfo = mock(SubtitleInfo.class);
        var subtitle = mock(Subtitle.class);
        var videoPlayback = mock(VideoPlayback.class);
        var subtitleFuture = new CompletableFuture<Subtitle>();
        var request = mock(PlayRequest.class);
        when(subtitleService.preference()).thenReturn(new SubtitlePreference(SubtitlePreferenceTag.LANGUAGE, SubtitleLanguage.POLISH));
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.ByReference.class))).thenReturn(CompletableFuture.completedFuture(subtitle));
        when(videoService.getVideoPlayer()).thenReturn(Optional.of(videoPlayback));
        when(videoPlayback.supportsNativeSubtitleFile()).thenReturn(false);
        when(request.getUrl()).thenReturn("http://localhost:9000/MyVideo.mp4");
        var service = new SubtitleManagerService(settingsService, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);
        var playbackListener = playbackListenerHolder.get();
        playbackListener.onPlay(request);
        service.registerListener(new SubtitleListener() {
            @Override
            public void onSubtitleChanged(Subtitle newSubtitle) {
                subtitleFuture.complete(newSubtitle);
            }

            @Override
            public void onSubtitleDisabled() {

            }
        });

        service.updateSubtitle(subtitleInfo);
        videoPlaybackProperty.set(videoPlayback);

        var result = subtitleFuture.get(200, TimeUnit.MILLISECONDS);
        assertEquals(subtitle, result);
    }
}
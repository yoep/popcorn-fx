package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.ApplicationSettingsEventListener;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import com.github.yoep.popcorn.backend.subtitles.SubtitleWrapper;
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
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class SubtitleManagerServiceTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private VideoService videoService;
    @Mock
    private ISubtitleService subtitleService;
    @Mock
    private SubtitlePickerService subtitlePickerService;
    @Mock
    private LocaleText localeText;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);

    private final AtomicReference<FxCallback<SubtitleEvent>> eventListenerHolder = new AtomicReference<>();
    private final AtomicReference<PlaybackListener> playbackListenerHolder = new AtomicReference<>();
    private final ObjectProperty<VideoPlayback> videoPlaybackProperty = new SimpleObjectProperty<>();
    private final AtomicReference<ApplicationSettingsEventListener> settingsEventListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder().build()));
        lenient().when(videoService.videoPlayerProperty()).thenReturn(videoPlaybackProperty);
        lenient().doAnswer(invocation -> {
            playbackListenerHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));
        lenient().doAnswer(invocation -> {
            eventListenerHolder.set(invocation.getArgument(0));
            return null;
        }).when(subtitleService).register(isA(FxCallback.class));
        doAnswer(invocations -> {
            settingsEventListenerHolder.set(invocations.getArgument(0, ApplicationSettingsEventListener.class));
            return null;
        }).when(applicationConfig).addListener(isA(ApplicationSettingsEventListener.class));
    }

    @Test
    void testUpdateSubtitleOffset_whenOffsetIsGiven_shouldUpdateTheOffsetValue() {
        var value = 100;
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

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
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        service.updateSubtitleOffset(value);

        verify(videoPlayer).subtitleDelay(value);
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsNone_shouldDisableSubtitleTrack() {
        var subtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(Subtitle.Language.NONE)
                .build());
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        service.updateSubtitle(subtitle);

        verify(subtitleService).disableSubtitle();
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsNull_shouldDisableSubtitleTrack() {
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        service.updateSubtitle(null);

        verify(subtitleService).disableSubtitle();
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsGivenAndNotCustom_shouldDownloadAndActivateTheSubtitle() {
        var language = Subtitle.Language.DUTCH;
        var subtitleInfo = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(language)
                .build());
        var request = Player.PlayRequest.newBuilder()
                .setUrl("http://localhost:9000/MyVideo.mp4")
                .build();
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);
        var listener = playbackListenerHolder.get();
        listener.onPlay(request);

        service.updateSubtitle(subtitleInfo);

        verify(subtitleService).updatePreferredLanguage(language);
    }

    @Test
    void testUpdateSubtitle_whenSubtitleDownloadFails_shouldPublishErrorNotification() {
        var language = Subtitle.Language.FINNISH;
        var expectedErrorText = "my error text";
        var subtitleInfo = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(language)
                .build());
        var request = Player.PlayRequest.newBuilder()
                .setUrl("http://localhost:9000/MyVideo.mp4")
                .build();
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(Subtitle.Matcher.class)))
                .thenReturn(CompletableFuture.failedFuture(new RuntimeException("my subtitle exception")));
        when(localeText.get(VideoMessage.SUBTITLE_DOWNLOAD_FILED)).thenReturn(expectedErrorText);
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);
        var listener = playbackListenerHolder.get();
        listener.onPlay(request);

        service.onSubtitleChanged(subtitleInfo);

        verify(eventPublisher).publishEvent(new ErrorNotificationEvent(service, expectedErrorText));
    }

    @Test
    void testUpdateSubtitle_whenSubtitleIsDownloadedAndVideoPlayerSupportNativeSubtitle_shouldUpdateSubtitleWithinVideoPlayer() {
        var language = Subtitle.Language.GERMAN;
        var subtitleInfo = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(language)
                .addFiles(Subtitle.Info.File.newBuilder().build())
                .build());
        var subtitle = new SubtitleWrapper(Subtitle.newBuilder()
                .setFilePath(".")
                .build());
        var videoPlayer = mock(VideoPlayback.class);
        var subtitleFile = new File(".");
        var request = Player.PlayRequest.newBuilder()
                .setUrl("http://localhost:9000/MyVideo.mp4")
                .build();
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(Subtitle.Matcher.class))).thenReturn(CompletableFuture.completedFuture(subtitle));
        when(videoService.getVideoPlayer()).thenReturn(Optional.of(videoPlayer));
        when(videoPlayer.supportsNativeSubtitleFile()).thenReturn(true);
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);
        var listener = playbackListenerHolder.get();
        listener.onPlay(request);

        service.onSubtitleChanged(subtitleInfo);

        verify(videoPlayer).subtitleFile(subtitleFile);
    }

    @Test
    void testSubtitleListener_whenSubtitleIsChangedToCustom_shouldLetTheUserPickASubtitle() {
        var request = Player.PlayRequest.newBuilder()
                .setUrl("http://localhost:9000/MyVideo.mp4")
                .build();
        var expected_filepath = "/lorem/ipsum.srt";
        when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.of(expected_filepath));
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);
        var playbackListener = playbackListenerHolder.get();
        playbackListener.onPlay(request);

        eventListenerHolder.get().callback(SubtitleEvent.newBuilder()
                .setEvent(SubtitleEvent.Event.PREFERENCE_CHANGED)
                .setPreferenceChanged(SubtitleEvent.PreferenceChanged.newBuilder()
                        .setPreference(SubtitlePreference.newBuilder()
                                .setPreference(SubtitlePreference.Preference.LANGUAGE)
                                .setLanguage(Subtitle.Language.CUSTOM)
                                .build())
                        .build())
                .build());

        verify(subtitleService).updatePreferredLanguage(Subtitle.Language.CUSTOM);
    }

    @Test
    void testSubtitleListener_whenSubtitleIsChangedToCustomAndUserCancels_shouldDisableTheSubtitleTrack() {
        var event = SubtitleEvent.newBuilder()
                .setEvent(SubtitleEvent.Event.PREFERENCE_CHANGED)
                .setPreferenceChanged(SubtitleEvent.PreferenceChanged.newBuilder()
                        .setPreference(SubtitlePreference.newBuilder()
                                .setPreference(SubtitlePreference.Preference.LANGUAGE)
                                .setLanguage(Subtitle.Language.CUSTOM)
                                .build())
                        .build())
                .build();
        when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.empty());
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        var listener = eventListenerHolder.get();
        listener.callback(event);

        verify(subtitleService).disableSubtitle();
    }

    @Test
    void testSubtitleSettingsListener_whenSubtitleSizeIsChangeD_shouldUpdateSubtitleSize() {
        var expectedValue = 34;
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setSubtitleSettings(ApplicationSettings.SubtitleSettings.newBuilder()
                        .setFontSize(12)
                        .build())
                .build()));
        var service = new SubtitleManagerService(applicationConfig, videoService, subtitleService, subtitlePickerService, localeText, eventPublisher);

        settingsEventListenerHolder.get().onSubtitleSettingsChanged(ApplicationSettings.SubtitleSettings.newBuilder()
                .setFontSize(expectedValue)
                .build());
        var result = service.getSubtitleSize();

        assertEquals(expectedValue, result);
    }
}
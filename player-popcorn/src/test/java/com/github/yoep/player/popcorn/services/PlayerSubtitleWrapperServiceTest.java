package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerSubtitleWrapperServiceTest {
    @Mock
    private VideoService videoService;
    @Mock
    private ISubtitleService subtitleService;
    @Mock
    private SubtitleManagerService subtitleManagerService;
    @Mock
    private PlayerSubtitleListener listener;

    private PlayerSubtitleService service;

    private final AtomicReference<PlaybackListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));
        when(subtitleService.defaultSubtitles()).thenReturn(CompletableFuture.completedFuture(asList(
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build()),
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build())
        )));

        service = new PlayerSubtitleService(videoService, subtitleService, subtitleManagerService);
        service.addListener(listener);
    }

    @Test
    void testUpdateSubtitleSizeWithSizeOffset_whenOffsetIsNegative_shouldDecreaseTheSubtitleSize() {
        var pixelChange = -5;
        var currentValue = 20;
        var expectedResult = currentValue + pixelChange;
        when(subtitleManagerService.getSubtitleSize()).thenReturn(currentValue);

        service.updateSubtitleSizeWithSizeOffset(pixelChange);

        verify(subtitleManagerService).setSubtitleSize(expectedResult);
    }

    @Test
    void testUpdateSubtitleSizeWithSizeOffset_whenOffsetIsPositive_shouldIncreaseTheSubtitleSize() {
        var pixelChange = 8;
        var currentValue = 24;
        var expectedResult = currentValue + pixelChange;
        when(subtitleManagerService.getSubtitleSize()).thenReturn(currentValue);

        service.updateSubtitleSizeWithSizeOffset(pixelChange);

        verify(subtitleManagerService).setSubtitleSize(expectedResult);
    }

    @Test
    void testUpdateActiveSubtitle_whenInvoked_shouldUpdateTheActiveSubtitle() {
        var subtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setImdbId("tt100000")
                .setLanguage(Subtitle.Language.ENGLISH)
                .build());

        service.updateActiveSubtitle(subtitle);

        verify(subtitleManagerService).updateSubtitle(subtitle);
    }

    @Test
    void testPlaybackListener_whenRequestIsShowPlayRequestWithoutSubtitle_shouldInvokeListenersWithAvailableEpisodeSubtitles() {
        var language = SubtitleLanguage.ENGLISH;
        var activeSubtitle = mock(SubtitleInfo.class);
        var request = mock(PlayRequest.class);
        var availableSubtitles = asList(mock(SubtitleInfo.class), mock(SubtitleInfo.class));
        when(request.getUrl()).thenReturn("http://localhost:8080/MyFilename.mp4");
        when(request.isSubtitlesEnabled()).thenReturn(true);
        when(request.getSubtitleInfo()).thenReturn(Optional.empty());
        when(subtitleService.preference()).thenReturn(new SubtitlePreference(SubtitlePreferenceTag.LANGUAGE, SubtitleLanguage.NONE));
        when(subtitleService.retrieveSubtitles(isA(String.class))).thenReturn(CompletableFuture.completedFuture(availableSubtitles));
        when(subtitleService.getDefaultOrInterfaceLanguage(availableSubtitles)).thenReturn(activeSubtitle);
        when(activeSubtitle.language()).thenReturn(language);

        listenerHolder.get().onPlay(request);

        verify(listener).onAvailableSubtitlesChanged(availableSubtitles, activeSubtitle);
        verify(subtitleService).retrieveSubtitles("MyFilename.mp4");
        verify(subtitleService).updatePreferredLanguage(language);
    }

    @Test
    void testPlaybackListener_whenRequestIsShowPlayRequestWithSubtitle_shouldInvokeListenersWithRequestSubtitle() {
        var language = SubtitleLanguage.SPANISH;
        var activeSubtitle = mock(SubtitleInfo.class);
        var request = mock(PlayRequest.class);
        var availableSubtitles = asList(mock(SubtitleInfo.class), mock(SubtitleInfo.class));
        when(request.getUrl()).thenReturn("http://localhost:8080/MyFilename.mp4");
        when(request.isSubtitlesEnabled()).thenReturn(true);
        when(request.getSubtitleInfo()).thenReturn(Optional.of(activeSubtitle));
        when(subtitleService.preference()).thenReturn(new SubtitlePreference(SubtitlePreferenceTag.LANGUAGE, language));
        when(subtitleService.retrieveSubtitles(isA(String.class))).thenReturn(CompletableFuture.completedFuture(availableSubtitles));

        listenerHolder.get().onPlay(request);

        verify(listener).onAvailableSubtitlesChanged(availableSubtitles, activeSubtitle);
        verify(subtitleService).retrieveSubtitles("MyFilename.mp4");
        verify(subtitleService, times(0)).getDefaultOrInterfaceLanguage(isA(List.class));
    }

    @Test
    void testPlaybackListener_whenRequestIsSimplePlayRequestAndSubtitlesIsDisabled_shouldNotRetrieveSubtitles() {
        var request = mock(PlayRequest.class);

        listenerHolder.get().onPlay(request);

        verify(subtitleService, times(0)).retrieveSubtitles(isA(String.class));
    }

    @Test
    void testPlaybackListener_whenRequestIsSimplePlayRequestAndSubtitlesIsEnabled_shouldInvokeListenersWithAvailableSubtitles() {
        var filename = "my-filename.mp4";
        var request = mock(PlayRequest.class);
        var availableSubtitles = asList(mock(SubtitleInfo.class), mock(SubtitleInfo.class));
        when(request.isSubtitlesEnabled()).thenReturn(true);
        when(request.getUrl()).thenReturn(filename);
        when(subtitleService.preference()).thenReturn(new SubtitlePreference(SubtitlePreferenceTag.LANGUAGE, SubtitleLanguage.NONE));
        when(subtitleService.retrieveSubtitles(filename)).thenReturn(CompletableFuture.completedFuture(availableSubtitles));
        when(subtitleService.getDefaultOrInterfaceLanguage(availableSubtitles)).thenReturn(subtitleNone);

        listenerHolder.get().onPlay(request);

        verify(listener).onAvailableSubtitlesChanged(availableSubtitles, subtitleNone);
    }

    @Test
    void testDefaultSubtitles() {
        var none = mock(SubtitleInfo.class);
        var custom = mock(SubtitleInfo.class);
        var expected = new SubtitleInfo[]{none, custom};
        when(subtitleService.none()).thenReturn(none);
        when(subtitleService.custom()).thenReturn(custom);

        var result = service.defaultSubtitles();

        assertArrayEquals(expected, result);
    }
}
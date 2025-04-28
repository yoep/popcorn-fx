package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.SubtitlePreference;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static java.util.Collections.singletonList;
import static org.junit.jupiter.api.Assertions.assertEquals;
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
        var language = Subtitle.Language.ENGLISH;
        var activeSubtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(language)
                .build());
        var request = Player.PlayRequest.newBuilder()
                .setUrl("http://localhost:8080/MyFilename.mp4")
                .setSubtitle(Player.PlayRequest.PlaySubtitleRequest.newBuilder()
                        .setEnabled(true)
                        .build())
                .build();
        List<ISubtitleInfo> availableSubtitles = createAvailableSubtitles();
        when(subtitleService.preference()).thenReturn(CompletableFuture.completedFuture(SubtitlePreference.newBuilder()
                .setPreference(SubtitlePreference.Preference.LANGUAGE)
                .setLanguage(Subtitle.Language.NONE)
                .build()
        ));
        when(subtitleService.retrieveSubtitles(isA(String.class))).thenReturn(CompletableFuture.completedFuture(availableSubtitles));
        when(subtitleService.getDefaultOrInterfaceLanguage(availableSubtitles)).thenReturn(CompletableFuture.completedFuture(activeSubtitle));

        listenerHolder.get().onPlay(request);

        verify(listener).onAvailableSubtitlesChanged(availableSubtitles, activeSubtitle);
        verify(subtitleService).retrieveSubtitles("MyFilename.mp4");
        verify(subtitleService).updatePreferredLanguage(language);
    }

    @Test
    void testPlaybackListener_whenRequestIsShowPlayRequestWithSubtitle_shouldInvokeListenersWithRequestSubtitle() {
        var language = Subtitle.Language.SPANISH;
        var activeSubtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(language)
                .build());
        var request = Player.PlayRequest.newBuilder()
                .setUrl("http://localhost:8080/MyFilename.mp4")
                .setSubtitle(Player.PlayRequest.PlaySubtitleRequest.newBuilder()
                        .setEnabled(true)
                        .setInfo(activeSubtitle.proto())
                        .build())
                .build();
        var defaultSubtitles = createAvailableSubtitles();
        List<ISubtitleInfo> availableSubtitles = singletonList( new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(Subtitle.Language.FINNISH)
                .build()));
        when(subtitleService.preference()).thenReturn(CompletableFuture.completedFuture(SubtitlePreference.newBuilder()
                .setPreference(SubtitlePreference.Preference.LANGUAGE)
                .setLanguage(language)
                .build()
        ));
        when(subtitleService.defaultSubtitles()).thenReturn(CompletableFuture.completedFuture(defaultSubtitles));
        when(subtitleService.retrieveSubtitles(isA(String.class))).thenReturn(CompletableFuture.completedFuture(availableSubtitles));

        listenerHolder.get().onPlay(request);

        verify(listener).onAvailableSubtitlesChanged(defaultSubtitles, defaultSubtitles.getFirst());
        verify(subtitleService, atLeastOnce()).defaultSubtitles();
        verify(subtitleService).retrieveSubtitles("MyFilename.mp4");
        verify(subtitleService, times(0)).getDefaultOrInterfaceLanguage(isA(List.class));
    }

    @Test
    void testPlaybackListener_whenRequestIsSimplePlayRequestAndSubtitlesIsDisabled_shouldNotRetrieveSubtitles() {
        var request = Player.PlayRequest.newBuilder()
                .setUrl("http://localhost:8080/MyFilename.mp4")
                .build();

        listenerHolder.get().onPlay(request);

        verify(subtitleService, times(0)).retrieveSubtitles(isA(String.class));
    }

    @Test
    void testPlaybackListener_whenRequestIsSimplePlayRequestAndSubtitlesIsEnabled_shouldInvokeListenersWithAvailableSubtitles() {
        var filename = "my-filename.mp4";
        var request = Player.PlayRequest.newBuilder()
                .setUrl(filename)
                .setSubtitle(Player.PlayRequest.PlaySubtitleRequest.newBuilder()
                        .setEnabled(true)
                        .build())
                .build();
        var subtitleNone = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(Subtitle.Language.NONE)
                .build());
        var availableSubtitles = createAvailableSubtitles();
        when(subtitleService.preference()).thenReturn(CompletableFuture.completedFuture(SubtitlePreference.newBuilder()
                .setPreference(SubtitlePreference.Preference.LANGUAGE)
                .setLanguage(Subtitle.Language.NONE)
                .build()
        ));
        when(subtitleService.retrieveSubtitles(filename)).thenReturn(CompletableFuture.completedFuture(availableSubtitles));
        when(subtitleService.getDefaultOrInterfaceLanguage(availableSubtitles)).thenReturn(CompletableFuture.completedFuture(subtitleNone));

        listenerHolder.get().onPlay(request);

        verify(listener, atLeastOnce()).onAvailableSubtitlesChanged(availableSubtitles, subtitleNone);
    }

    @Test
    void testDefaultSubtitles() {
        List<ISubtitleInfo> defaultSubtitles = asList(
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build()),
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build())
        );
        when(subtitleService.defaultSubtitles()).thenReturn(CompletableFuture.completedFuture(defaultSubtitles));

        var result = service.defaultSubtitles().resultNow();

        assertEquals(defaultSubtitles, result);
    }

    private static List<ISubtitleInfo> createAvailableSubtitles() {
        return asList(
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build()),
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build())
        );
    }
}
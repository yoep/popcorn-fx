package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.player.model.MediaPlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerSubtitleServiceTest {
    @Mock
    private VideoService videoService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private SubtitleManagerService subtitleManagerService;
    @Mock
    private PlayerSubtitleListener listener;
    @InjectMocks
    private PlayerSubtitleService service;

    private final ObjectProperty<Subtitle> subtitleProperty = new SimpleObjectProperty<>();
    private final AtomicReference<PlaybackListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));
        lenient().when(subtitleService.activeSubtitleProperty()).thenReturn(subtitleProperty);

        service.addListener(listener);
    }

    @Test
    void testUpdateSubtitleSizeWithSizeOffset_whenInvoked_shouldUpdateTheSubtitleSize() {
        var pixelChange = 80;

        service.updateSubtitleSizeWithSizeOffset(pixelChange);

        verify(subtitleManagerService).updateSubtitleOffset(pixelChange);
    }

    @Test
    void testUpdateActiveSubtitle_whenInvoked_shouldUpdateTheActiveSubtitle() {
        var subtitle = mock(SubtitleInfo.class);

        service.updateActiveSubtitle(subtitle);

        verify(subtitleManagerService).updateSubtitle(subtitle);
    }

    @Test
    void testSubtitlePropertyListener_whenChanged_shouldInvokedListeners() {
        var subtitle = mock(Subtitle.class);
        var subtitleInfo = mock(SubtitleInfo.class);
        when(subtitle.getSubtitleInfo()).thenReturn(Optional.of(subtitleInfo));
        service.init();

        subtitleProperty.set(subtitle);

        verify(listener).onActiveSubtitleChanged(subtitleInfo);
    }

    @Test
    void testPlaybackListener_whenRequestIsMediaPlayRequest_shouldInvokeListenersWithAvailableSubtitles() {
        var movie = Movie.builder().build();
        var activeSubtitle = mock(SubtitleInfo.class);
        var request = MediaPlayRequest.mediaBuilder()
                .media(movie)
                .subtitle(activeSubtitle)
                .build();
        var availableSubtitles = asList(mock(SubtitleInfo.class), mock(SubtitleInfo.class));
        when(subtitleService.retrieveSubtitles(movie)).thenReturn(CompletableFuture.completedFuture(availableSubtitles));
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onAvailableSubtitlesChanged(availableSubtitles, activeSubtitle);
    }

    @Test
    void testPlaybackListener_whenRequestIsSimplePlayRequestAndSubtitlesIsDisabled_shouldNotRetrieveSubtitles() {
        var request = SimplePlayRequest.builder()
                .title("lorem")
                .url("filename.mp4")
                .build();
        service.init();

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
        when(subtitleService.retrieveSubtitles(filename)).thenReturn(CompletableFuture.completedFuture(availableSubtitles));
        when(subtitleService.getDefault(availableSubtitles)).thenReturn(SubtitleInfo.none());
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onAvailableSubtitlesChanged(availableSubtitles, SubtitleInfo.none());
    }
}
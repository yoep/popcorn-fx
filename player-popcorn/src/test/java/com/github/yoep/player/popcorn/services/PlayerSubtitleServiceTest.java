package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.player.model.MediaPlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
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
    @Mock
    private FxLib fxLib;
    @Mock
    private SubtitleInfo subtitleNone;
    @InjectMocks
    private PlayerSubtitleService service;

    private final AtomicReference<PlaybackListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlaybackListener.class));
            return null;
        }).when(videoService).addListener(isA(PlaybackListener.class));
        lenient().when(fxLib.subtitle_none()).thenReturn(subtitleNone);

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
        var subtitle = mock(SubtitleInfo.class);

        service.updateActiveSubtitle(subtitle);

        verify(subtitleManagerService).updateSubtitle(subtitle);
    }

    @Test
    void testPlaybackListener_whenRequestIsMoviePlayRequest_shouldInvokeListenersWithAvailableSubtitles() {
        var movie = MovieDetails.builder().build();
        var activeSubtitle = mock(SubtitleInfo.class);
        var torrentStream = mock(TorrentStream.class);
        var request = MediaPlayRequest.mediaBuilder()
                .media(movie)
                .torrentStream(torrentStream)
                .build();
        var availableSubtitles = asList(mock(SubtitleInfo.class), mock(SubtitleInfo.class));
        when(subtitleService.retrieveSubtitles(movie)).thenReturn(CompletableFuture.completedFuture(availableSubtitles));
        when(subtitleService.preferredSubtitle()).thenReturn(Optional.of(activeSubtitle));
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onAvailableSubtitlesChanged(availableSubtitles, activeSubtitle);
    }

    @Test
    void testPlaybackListener_whenRequestIsShowPlayRequest_shouldInvokeListenersWithAvailableEpisodeSubtitles() {
        var episode = Episode.builder()
                .episode(2)
                .build();
        var show = mock(ShowDetails.class);
        var activeSubtitle = mock(SubtitleInfo.class);
        var torrentStream = mock(TorrentStream.class);
        var request = MediaPlayRequest.mediaBuilder()
                .media(show)
                .subMediaItem(episode)
                .torrentStream(torrentStream)
                .build();
        var availableSubtitles = asList(mock(SubtitleInfo.class), mock(SubtitleInfo.class));
        when(subtitleService.retrieveSubtitles(show, episode)).thenReturn(CompletableFuture.completedFuture(availableSubtitles));
        when(subtitleService.preferredSubtitle()).thenReturn(Optional.of(activeSubtitle));
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
        service.init();

        listenerHolder.get().onPlay(request);

        verify(listener).onAvailableSubtitlesChanged(availableSubtitles, subtitleNone);
    }
}
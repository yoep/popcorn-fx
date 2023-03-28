package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoTorrentEvent;
import com.github.yoep.popcorn.backend.media.providers.models.*;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.TorrentSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.ui.events.CloseLoadEvent;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.LoadUrlTorrentEvent;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.File;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class LoadTorrentServiceTest {
    @Mock
    private TorrentService torrentService;
    @Mock
    private TorrentStreamService torrentStreamService;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ApplicationConfig settingsService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private TorrentSettings torrentSettings;
    @InjectMocks
    private LoadTorrentService service;
    @TempDir
    File workingDir;

    @BeforeEach
    void setUp() {
        lenient().when(settingsService.getSettings()).thenReturn(settings);
        lenient().when(settings.getTorrentSettings()).thenReturn(torrentSettings);
        lenient().when(torrentSettings.getDirectory()).thenReturn(workingDir.getAbsolutePath());
    }

    @Test
    void testCancel_whenInvoked_shouldPublishCloseEvent() {
        var future = mock(CompletableFuture.class);
        var torrentMagnet = "magnet://my-cancelled-torrent";
        var event = LoadMediaTorrentEvent.builder()
                .source(this)
                .torrent(MediaTorrentInfo.builder()
                        .url(torrentMagnet)
                        .build())
                .quality("720p")
                .media(MovieDetails.builder()
                        .images(new Images())
                        .build())
                .build();
        when(torrentService.getSessionState()).thenReturn(SessionState.RUNNING);
        when(torrentService.getTorrentInfo(torrentMagnet)).thenReturn(future);
        when(future.thenCompose(isA(Function.class))).thenReturn(future);
        when(future.exceptionally(isA(Function.class))).thenReturn(future);
        service.init();

        eventPublisher.publish(event);
        service.cancel();

        verify(eventPublisher).publishEvent(new CloseLoadEvent(service));
        verify(future).cancel(true);
    }

    @Test
    void testRetryLoadingTorrent_whenPreviousLoadFailed_shouldPublishEventAgain() {
        var torrentMagnet = "magnet://my-retrying-torrent";
        var event = LoadMediaTorrentEvent.builder()
                .source(this)
                .torrent(MediaTorrentInfo.builder()
                        .url(torrentMagnet)
                        .build())
                .quality("720p")
                .media(MovieDetails.builder()
                        .images(new Images())
                        .build())
                .build();
        when(torrentService.getSessionState()).thenReturn(SessionState.RUNNING);
        when(torrentService.getTorrentInfo(torrentMagnet)).thenReturn(new CompletableFuture<>());
        service.init();

        eventPublisher.publish(event);
        service.retryLoadingTorrent();

        verify(eventPublisher).publishEvent(event);
    }

    @Test
    void testOnLoadMediaTorrent_whenDefaultSubtitleIsAvailable_shouldActivateSubtitle() {
        var torrentInfo = mock(TorrentInfo.class);
        var torrent = mock(Torrent.class);
        var torrentFileInfo = mock(TorrentFileInfo.class);
        var torrentMagnet = "magnet://my-show-torrent";
        var episodeTitle = "episode-001";
        var torrentFilename = episodeTitle + ".mkv";
        var quality = "1080p";
        var episode = Episode.builder()
                .title(episodeTitle)
                .tvdbId("tv-id-001254")
                .build();
        var media = mock(ShowDetails.class);
        var event = LoadMediaTorrentEvent.builder()
                .source(this)
                .torrent(MediaTorrentInfo.builder()
                        .url(torrentMagnet)
                        .file(episodeTitle)
                        .build())
                .quality(quality)
                .media(media)
                .subItem(episode)
                .build();
        var subtitleInfo = SubtitleInfo.builder()
                .imdbId("myImdbId")
                .language(SubtitleLanguage.ENGLISH)
                .build();
        var subtitleMatcher = SubtitleMatcher.from(torrentFilename, quality);
        when(media.getTitle()).thenReturn("my show title");
        when(torrentInfo.getByFilename(episodeTitle)).thenReturn(Optional.of(torrentFileInfo));
        when(torrent.getFilename()).thenReturn(torrentFilename);
        when(torrentService.getSessionState()).thenReturn(SessionState.RUNNING);
        when(torrentService.getTorrentInfo(torrentMagnet)).thenReturn(CompletableFuture.completedFuture(torrentInfo));
        when(torrentService.create(torrentFileInfo, workingDir, true)).thenReturn(CompletableFuture.completedFuture(torrent));
        when(torrentStreamService.startStream(isA(Torrent.class))).thenReturn(mock(TorrentStream.class));
        when(subtitleService.download(subtitleInfo, subtitleMatcher)).thenReturn(CompletableFuture.completedFuture(""));
        when(subtitleService.preferredSubtitleLanguage()).thenReturn(SubtitleLanguage.ENGLISH);
        when(subtitleService.preferredSubtitle()).thenReturn(Optional.of(subtitleInfo));
        service.init();

        eventPublisher.publish(event);

        verify(subtitleService).download(subtitleInfo, subtitleMatcher);
    }

    @Test
    void testOnLoadMediaTorrent_whenStreamIsReady_shouldInvokePlayMediaEvent() {
        var listenerHolder = new AtomicReference<TorrentStreamListener>();
        var torrentInfo = mock(TorrentInfo.class);
        var torrent = mock(Torrent.class);
        var torrentStream = mock(TorrentStream.class);
        var torrentFileInfo = mock(TorrentFileInfo.class);
        var quality = "720";
        var title = "my-movie-title";
        var torrentMagnet = "magnet://my-torrent";
        var torrentFile = "my-file";
        var media = MovieDetails.builder()
                .title(title)
                .images(Images.builder().build())
                .build();
        var event = LoadMediaTorrentEvent.builder()
                .source(this)
                .torrent(MediaTorrentInfo.builder()
                        .url(torrentMagnet)
                        .file(torrentFile)
                        .build())
                .quality(quality)
                .media(media)
                .build();
        var expectedMediaEvent = PlayMediaEvent.mediaBuilder()
                .source(service)
                .media(media)
                .torrent(torrent)
                .torrentStream(torrentStream)
                .title(title)
                .quality(quality)
                .url("")
                .build();
        when(torrentInfo.getByFilename(torrentFile)).thenReturn(Optional.of(torrentFileInfo));
        when(torrentStream.getStreamUrl()).thenReturn("http:my-stream-url");
        when(torrentService.getSessionState()).thenReturn(SessionState.RUNNING);
        when(torrentService.getTorrentInfo(torrentMagnet)).thenReturn(CompletableFuture.completedFuture(torrentInfo));
        when(torrentService.create(torrentFileInfo, workingDir, true)).thenReturn(CompletableFuture.completedFuture(torrent));
        when(torrentStreamService.startStream(torrent)).thenReturn(torrentStream);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, TorrentStreamListener.class));
            return null;
        }).when(torrentStream).addListener(isA(TorrentStreamListener.class));
        service.init();

        eventPublisher.publish(event);
        listenerHolder.get().onStreamReady();

        verify(eventPublisher).publishEvent(expectedMediaEvent);
    }

    @Test
    void testLoadMediaTorrent_shouldUpdatePreferredSubtitle() {
        var subtitleInfo = mock(SubtitleInfo.class);
        var event = LoadMediaTorrentEvent.builder()
                .source(this)
                .torrent(mock(MediaTorrentInfo.class))
                .quality("1080p")
                .media(mock(Media.class))
                .subtitle(subtitleInfo)
                .build();
        service.init();

        eventPublisher.publish(event);

        verify(subtitleService).updateSubtitle(subtitleInfo);
    }

    @Test
    void testLoadTorrentUrl_whenUrlIsGiven_shouldInvokePlayVideoEvent() {
        var listenerHolder = new AtomicReference<TorrentStreamListener>();
        var filename = "lorem ipsum.mkv";
        var url = "http://localhost:8081/" + filename;
        var torrentInfo = mock(TorrentInfo.class);
        var torrentFileInfo = mock(TorrentFileInfo.class);
        var torrent = mock(Torrent.class);
        var torrentStream = mock(TorrentStream.class);
        var subtitleInfo = SubtitleInfo.builder()
                .imdbId("tv00001")
                .language(SubtitleLanguage.ENGLISH)
                .build();
        var event = LoadUrlTorrentEvent.builder()
                .source(this)
                .torrentInfo(torrentInfo)
                .torrentFileInfo(torrentFileInfo)
                .build();
        var subtitleMatcher = SubtitleMatcher.from(filename, null);
        var expectedResult = PlayVideoTorrentEvent.videoTorrentBuilder()
                .source(service)
                .url(url)
                .title(filename)
                .torrent(torrent)
                .torrentStream(torrentStream)
                .subtitlesEnabled(true)
                .build();
        when(torrentService.getSessionState()).thenReturn(SessionState.RUNNING);
        when(torrentService.create(torrentFileInfo, workingDir, true)).thenReturn(CompletableFuture.completedFuture(torrent));
        when(torrentStreamService.startStream(torrent)).thenReturn(torrentStream);
        when(subtitleService.preferredSubtitleLanguage()).thenReturn(SubtitleLanguage.ENGLISH);
        when(subtitleService.preferredSubtitle()).thenReturn(Optional.of(subtitleInfo));
        when(subtitleService.download(isA(SubtitleInfo.class), isA(SubtitleMatcher.class))).thenReturn(CompletableFuture.completedFuture(""));
        when(torrentFileInfo.getFilename()).thenReturn(filename);
        when(torrentStream.getStreamUrl()).thenReturn(url);
        when(torrent.getFilename()).thenReturn(filename);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, TorrentStreamListener.class));
            return null;
        }).when(torrentStream).addListener(isA(TorrentStreamListener.class));
        service.init();

        eventPublisher.publish(event);
        listenerHolder.get().onStreamReady();

        verify(eventPublisher).publishEvent(expectedResult);
        verify(subtitleService).download(subtitleInfo, subtitleMatcher);
    }
}
package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoTorrentEvent;
import com.github.yoep.popcorn.backend.media.providers.models.*;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.TorrentSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
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
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import java.io.File;
import java.util.Collections;
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
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private SettingsService settingsService;
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
        lenient().when(torrentSettings.getDirectory()).thenReturn(workingDir);
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
                .media(Movie.builder().build())
                .build();
        when(torrentService.getSessionState()).thenReturn(SessionState.RUNNING);
        when(torrentService.getTorrentInfo(torrentMagnet)).thenReturn(future);
        when(future.thenCompose(isA(Function.class))).thenReturn(future);
        when(future.exceptionally(isA(Function.class))).thenReturn(future);

        service.onLoadMediaTorrent(event);
        service.cancel();

        verify(eventPublisher).publishEvent(new CloseLoadEvent(service));
        verify(subtitleService).setActiveSubtitle(Subtitle.none());
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
                .media(Movie.builder().build())
                .build();
        when(torrentService.getSessionState()).thenReturn(SessionState.RUNNING);
        when(torrentService.getTorrentInfo(torrentMagnet)).thenReturn(new CompletableFuture<>());

        service.onLoadMediaTorrent(event);
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
        var media = Show.builder()
                .title("my show title")
                .episodes(Collections.singletonList(episode))
                .images(Images.builder().build())
                .build();
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
        var subtitle = mock(Subtitle.class);
        var subtitleMatcher = SubtitleMatcher.from(torrentFilename, quality);
        when(torrentInfo.getByFilename(episodeTitle)).thenReturn(Optional.of(torrentFileInfo));
        when(torrent.getFilename()).thenReturn(torrentFilename);
        when(torrentService.getSessionState()).thenReturn(SessionState.RUNNING);
        when(torrentService.getTorrentInfo(torrentMagnet)).thenReturn(CompletableFuture.completedFuture(torrentInfo));
        when(torrentService.create(torrentFileInfo, workingDir, true)).thenReturn(CompletableFuture.completedFuture(torrent));
        when(subtitleService.retrieveSubtitles(media, episode)).thenReturn(CompletableFuture.completedFuture(Collections.singletonList(subtitleInfo)));
        when(subtitleService.getDefaultOrInterfaceLanguage(Collections.singletonList(subtitleInfo))).thenReturn(subtitleInfo);
        when(subtitleService.downloadAndParse(subtitleInfo, subtitleMatcher)).thenReturn(CompletableFuture.completedFuture(subtitle));

        service.onLoadMediaTorrent(event);

        verify(subtitleService).setActiveSubtitle(subtitle);
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
        var media = Movie.builder()
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
        when(subtitleService.retrieveSubtitles(media)).thenReturn(CompletableFuture.completedFuture(Collections.emptyList()));
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, TorrentStreamListener.class));
            return null;
        }).when(torrentStream).addListener(isA(TorrentStreamListener.class));

        service.onLoadMediaTorrent(event);
        listenerHolder.get().onStreamReady();

        verify(eventPublisher).publishEvent(expectedMediaEvent);
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
        var subtitle = new Subtitle(subtitleInfo, new File(""), Collections.emptyList());
        var availableSubtitles = Collections.singletonList(subtitleInfo);
        var event = LoadUrlTorrentEvent.builder()
                .source(this)
                .torrentInfo(torrentInfo)
                .torrentFileInfo(torrentFileInfo)
                .build();
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
        when(subtitleService.retrieveSubtitles(isA(String.class))).thenReturn(CompletableFuture.completedFuture(availableSubtitles));
        when(subtitleService.getDefaultOrInterfaceLanguage(availableSubtitles)).thenReturn(subtitleInfo);
        when(subtitleService.downloadAndParse(subtitleInfo, SubtitleMatcher.from(filename, (String) null))).thenReturn(CompletableFuture.completedFuture(subtitle));
        when(torrentFileInfo.getFilename()).thenReturn(filename);
        when(torrentStream.getStreamUrl()).thenReturn(url);
        when(torrent.getFilename()).thenReturn(filename);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, TorrentStreamListener.class));
            return null;
        }).when(torrentStream).addListener(isA(TorrentStreamListener.class));

        service.onLoadUrlTorrent(event);
        listenerHolder.get().onStreamReady();

        verify(eventPublisher).publishEvent(expectedResult);
        verify(subtitleService).retrieveSubtitles(filename);
        verify(subtitleService).setActiveSubtitle(subtitle);
    }
}
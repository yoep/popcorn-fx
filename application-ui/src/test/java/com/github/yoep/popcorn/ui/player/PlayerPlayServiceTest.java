package com.github.yoep.popcorn.ui.player;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoTorrentEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.resume.AutoResumeService;
import com.github.yoep.popcorn.backend.player.model.MediaPlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import com.github.yoep.popcorn.backend.player.model.StreamPlayRequest;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.PlaybackSettings;
import com.github.yoep.popcorn.ui.messages.MediaMessage;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerPlayServiceTest {
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private AutoResumeService autoResumeService;
    @Mock
    private ScreenService screenService;
    @Mock
    private SettingsService settingsService;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private PlaybackSettings playbackSettings;
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private LocaleText localeText;
    @InjectMocks
    private PlayerPlayService service;

    @BeforeEach
    void setUp() {
        lenient().when(settingsService.getSettings()).thenReturn(settings);
    }

    @Test
    void testOnPlayVideo_whenThereIsNoActivePlayer_shouldNotThrowAnException() {
        var event = mock(PlayVideoEvent.class);
        when(event.getUrl()).thenReturn("my play video event url");
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.empty());

        service.onPlayVideo(event);

        assertTrue(true);
    }

    @Test
    void testOnPlayVideo_whenPlayerIsActive_shouldInvokedPlayOnThePlayer() {
        var url = "my video url";
        var title = "my video title";
        var event = new PlayVideoEvent(this, url, title, true);
        var player = mock(Player.class);
        var expectedResult = SimplePlayRequest.builder()
                .url(url)
                .title(title)
                .build();
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(settings.getPlaybackSettings()).thenReturn(playbackSettings);

        service.onPlayVideo(event);

        verify(player).play(expectedResult);
    }

    @Test
    void testOnPlayVideo_whenFullscreenPlaybackIsEnabled_shouldActiveFullscreenMode() {
        var event = mock(PlayVideoEvent.class);
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(settings.getPlaybackSettings()).thenReturn(playbackSettings);
        when(playbackSettings.isFullscreen()).thenReturn(true);

        service.onPlayVideo(event);

        verify(screenService).fullscreen(true);
    }

    @Test
    void testOnPlayVideo_whenEventIsPlayMediaEvent_shouldInvokeMediaPlayRequestOnPlayer() {
        var id = "tt006";
        var url = "my-media-video.mp4";
        var title = "006";
        var timestamp = 18000L;
        var player = mock(Player.class);
        var media = Movie.builder()
                .id(id)
                .images(Images.builder().build())
                .build();
        var torrent = mock(Torrent.class);
        var torrentStream = mock(TorrentStream.class);
        var event = PlayMediaEvent.mediaBuilder()
                .source(this)
                .url(url)
                .title(title)
                .media(media)
                .torrent(torrent)
                .torrentStream(torrentStream)
                .build();
        var request = MediaPlayRequest.mediaBuilder()
                .url(url)
                .title(title)
                .media(media)
                .autoResumeTimestamp(timestamp)
                .torrentStream(torrentStream)
                .build();
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(settings.getPlaybackSettings()).thenReturn(playbackSettings);
        when(playbackSettings.isFullscreen()).thenReturn(false);
        when(autoResumeService.getResumeTimestamp(id, url)).thenReturn(Optional.of(timestamp));

        service.onPlayVideo(event);

        verify(player).play(request);
    }

    @Test
    void testOnPlayVideo_whenEventIsPlayTorrentVideoEvent_shouldInvokeStreamPlayRequestOnPlayer() {
        var url = "my-torrent-video.mp4";
        var title = "001";
        var timestamp = 16000L;
        var player = mock(Player.class);
        var torrent = mock(Torrent.class);
        var torrentStream = mock(TorrentStream.class);
        var event = PlayVideoTorrentEvent.videoTorrentBuilder()
                .source(this)
                .url(url)
                .title(title)
                .torrent(torrent)
                .torrentStream(torrentStream)
                .build();
        var request = StreamPlayRequest.streamBuilder()
                .url(url)
                .title(title)
                .autoResumeTimestamp(timestamp)
                .torrentStream(torrentStream)
                .build();
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(settings.getPlaybackSettings()).thenReturn(playbackSettings);
        when(playbackSettings.isFullscreen()).thenReturn(false);
        when(autoResumeService.getResumeTimestamp(url)).thenReturn(Optional.of(timestamp));

        service.onPlayVideo(event);

        verify(player).play(request);
    }

    @Test
    void testOnPlayVideo_whenEventIsPlayVideoEvent_shouldInvokeSimplePlayRequestOnPlayer() {
        var url = "my-video.mp4";
        var title = "file-video";
        var timestamp = 22000L;
        var player = mock(Player.class);
        var event = PlayVideoEvent.builder()
                .source(this)
                .url(url)
                .title(title)
                .build();
        var request = SimplePlayRequest.builder()
                .url(url)
                .title(title)
                .autoResumeTimestamp(timestamp)
                .build();
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(settings.getPlaybackSettings()).thenReturn(playbackSettings);
        when(playbackSettings.isFullscreen()).thenReturn(false);
        when(autoResumeService.getResumeTimestamp(url)).thenReturn(Optional.of(timestamp));

        service.onPlayVideo(event);

        verify(player).play(request);
    }

    @Test
    void testOnPlayVideo_whenPlayerThrowsError_shouldShowErrorNotification() {
        var url = "my-video.mp4";
        var title = "file-video";
        var errorMessage = "video player failed";
        var player = mock(Player.class);
        var event = PlayVideoEvent.builder()
                .source(this)
                .url(url)
                .title(title)
                .build();
        var request = SimplePlayRequest.builder()
                .url(url)
                .title(title)
                .build();
        var expectedErrorNotification = new ErrorNotificationEvent(service, errorMessage);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));
        when(autoResumeService.getResumeTimestamp(url)).thenReturn(Optional.empty());
        when(localeText.get(MediaMessage.VIDEO_PLAYBACK_FAILED)).thenReturn(errorMessage);
        doThrow(new RuntimeException("my player error")).when(player).play(request);

        service.onPlayVideo(event);

        verify(eventPublisher).publishEvent(expectedErrorNotification);
    }
}

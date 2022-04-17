package com.github.yoep.popcorn.ui.playnext;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayTorrentEvent;
import com.github.yoep.popcorn.backend.media.providers.models.*;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.PlaybackSettings;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.player.PlayerEventService;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import java.util.Collections;
import java.util.HashMap;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayNextServiceTest {
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private PlayerEventService playerEventService;
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private SettingsService settingsService;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private PlaybackSettings playbackSettings;
    @InjectMocks
    private PlayNextService playNextService;

    private final AtomicReference<PlayerListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().when(settingsService.getSettings()).thenReturn(settings);
        lenient().when(settings.getPlaybackSettings()).thenReturn(playbackSettings);
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(playerEventService).addListener(isA(PlayerListener.class));
    }

    @Test
    void testOnPlayVideo_whenEventIsPlayTorrentEvent_shouldNotUpdateNextEpisode() {
        var activity = mock(PlayTorrentEvent.class);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);

        assertTrue(playNextService.getNextEpisode().isEmpty());
    }

    @Test
    void testOnPlayMedia_whenMediaIsMovie_shouldNotUpdateNextEpisode() {
        var activity = mock(PlayMediaEvent.class);
        when(activity.getMedia()).thenReturn(mock(Movie.class));
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);

        assertTrue(playNextService.getNextEpisode().isEmpty());
    }

    @Test
    void testOnPlayMedia_whenMediaIsEpisodeAndPlayNextIsDisabled_shouldNotUpdateNextEpisode() {
        var activity = mock(PlayMediaEvent.class);
        var episode = mock(Episode.class);
        lenient().when(activity.getMedia()).thenReturn(episode);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(false);

        playNextService.onPlayVideo(activity);

        assertTrue(playNextService.getNextEpisode().isEmpty());
    }

    @Test
    void testOnPlayMedia_whenMediaIsEpisodeAndPlayNextIsEnabled_shouldUpdateNextEpisode() {
        var episode = Episode.builder()
                .season(1)
                .episode(6)
                .build();
        var show = Show.builder()
                .images(Images.builder().build())
                .episodes(Collections.singletonList(episode))
                .build();
        var activity = PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("my-url")
                .title("my-title")
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .media(show)
                .subMediaItem(Episode.builder()
                        .episode(5)
                        .build())
                .build();
        var expectedResult = new PlayNextService.NextEpisode(show, episode);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);
        var result = playNextService.getNextEpisode();

        assertTrue(result.isPresent());
        assertEquals(expectedResult, result.get());
    }

    @Test
    void testOnPlayMedia_whenEpisodeIsLastEpisodeInShow_shouldNotUpdateNextEpisode() {
        var episode1 = createEpisode(1);
        var episode2 = createEpisode(2);
        var show = Show.builder()
                .images(Images.builder().build())
                .episodes(asList(episode1, episode2))
                .build();
        var activity = PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("my-url")
                .title("my-title")
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .media(show)
                .subMediaItem(episode2)
                .build();
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);
        var result = playNextService.getNextEpisode();

        assertTrue(result.isEmpty());
    }

    @Test
    void testOnPlayMedia_whenEpisodesAreNotInOrder_shouldSelectTheCorrectNextEpisode() {
        var episode1 = createEpisode(1);
        var episode2 = createEpisode(2);
        var episode3 = createEpisode(3);
        var episode4 = createEpisode(4);
        var show = Show.builder()
                .images(Images.builder().build())
                .episodes(asList(episode1, episode2, episode3, episode4))
                .build();
        var activity = PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("my-url")
                .title("my-title")
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .media(show)
                .subMediaItem(episode2)
                .build();
        show.setEpisodes(asList(episode1, episode3, episode2, episode4));
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);
        var result = playNextService.getNextEpisode();

        assertTrue(result.isPresent(), "next episode should be available");
        assertEquals(show, result.get().getShow());
        assertEquals(episode3, result.get().getEpisode());
    }

    //region OnTimeChanged

    @Test
    void testOnTimeChanged_whenPlayNextIsDisabled_shouldNotUpdatePlayingIn() {
        var episode = createEpisode(1);
        var nextEpisode = createEpisode(2);
        var show = Show.builder()
                .images(Images.builder().build())
                .episodes(asList(episode, nextEpisode))
                .build();
        var activity = PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("my-url")
                .title("my-title")
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .media(show)
                .subMediaItem(episode)
                .build();
        playNextService.init();

        // update the next episode
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);
        playNextService.onPlayVideo(activity);

        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(false);
        listenerHolder.get().onDurationChanged(90);
        listenerHolder.get().onTimeChanged(70);
        var result = playNextService.getPlayingIn();

        assertEquals(PlayNextService.COUNTDOWN_FROM, result);
    }

    @Test
    void testOnTimeChanged_whenPlayNextIsEnabled_shouldUpdatePlayingInValue() {
        var episode = createEpisode(1);
        var nextEpisode = createEpisode(2);
        var show = Show.builder()
                .images(Images.builder().build())
                .episodes(asList(episode, nextEpisode))
                .build();
        var expectedResult = 20;
        var activity = PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("my-url")
                .title("my-title")
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .media(show)
                .subMediaItem(episode)
                .build();
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);
        playNextService.init();

        playNextService.onPlayVideo(activity);
        listenerHolder.get().onDurationChanged(90000);
        listenerHolder.get().onTimeChanged(70000);
        var result = playNextService.getPlayingIn();

        assertEquals(expectedResult, result);
    }

    @Test
    void testOnTimeChanged_whenPlayNextIsEnabledAndRemainingTimeIsZero_shouldTriggerTheNextEpisodePlayback() {
        var episode1 = createEpisode(1);
        var episode2 = createEpisode(2);
        var show = Show.builder()
                .images(Images.builder().build())
                .episodes(asList(episode1, episode2))
                .build();
        var activity = PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("my-url")
                .title("my-title")
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .media(show)
                .subMediaItem(episode1)
                .quality("480p")
                .build();
        var videoLength = 90000;
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);
        playNextService.init();

        playNextService.onPlayVideo(activity);
        listenerHolder.get().onDurationChanged(videoLength);
        listenerHolder.get().onTimeChanged(videoLength);

        verify(eventPublisher).publishEvent(isA(LoadMediaTorrentEvent.class));
    }

    @Test
    void testOnTimeChanged_whenMediaTimeAndDurationInfoIsNotKnown_shouldNotTriggerTheNextEpisodePlayback() {
        var episode1 = createEpisode(1);
        var episode2 = createEpisode(2);
        var show = Show.builder()
                .images(Images.builder().build())
                .episodes(asList(episode1, episode2))
                .build();
        var activity = PlayMediaEvent.mediaBuilder()
                .source(this)
                .url("my-url")
                .title("my-title")
                .torrent(mock(Torrent.class))
                .torrentStream(mock(TorrentStream.class))
                .media(show)
                .subMediaItem(episode1)
                .quality("480p")
                .build();
        var videoLength = 0;
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);
        playNextService.init();

        playNextService.onPlayVideo(activity);
        listenerHolder.get().onDurationChanged(videoLength);
        listenerHolder.get().onTimeChanged(videoLength);

        verify(eventPublisher, times(0)).publishEvent(isA(LoadMediaTorrentEvent.class));
    }

    @Test
    void testStop_whenInvoked_shouldStopThePlayer() {
        var player = mock(Player.class);
        when(playerManagerService.getActivePlayer()).thenReturn(Optional.of(player));

        playNextService.stop();

        verify(player).stop();
    }

    //endregion

    //endregion

    private Episode createEpisode(int number) {
        var torrents = new HashMap<String, MediaTorrentInfo>();
        var episode = Episode.builder()
                .episode(number)
                .torrents(torrents)
                .build();

        torrents.put("480p", mock(MediaTorrentInfo.class));
        torrents.put("720p", mock(MediaTorrentInfo.class));

        return episode;
    }
}

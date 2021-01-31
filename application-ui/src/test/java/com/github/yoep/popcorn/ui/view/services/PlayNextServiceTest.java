package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.PlayMediaEvent;
import com.github.yoep.popcorn.ui.events.PlayTorrentEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.ui.settings.models.PlaybackSettings;
import com.github.yoep.popcorn.ui.view.listeners.VideoPlayerListener;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;

import java.util.HashMap;
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
    private VideoPlayerService videoPlayerService;
    @Mock
    private SettingsService settingsService;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private PlaybackSettings playbackSettings;
    @InjectMocks
    private PlayNextService playNextService;

    @BeforeEach
    void setUp() {
        lenient().when(settingsService.getSettings()).thenReturn(settings);
        lenient().when(settings.getPlaybackSettings()).thenReturn(playbackSettings);
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
        var activity = mock(PlayMediaEvent.class);
        var show = Show.builder().build();
        var episode = Episode.builder()
                .season(1)
                .episode(6)
                .show(show)
                .build();
        var expectedResult = Episode.builder()
                .season(2)
                .episode(1)
                .show(show)
                .build();
        show.setEpisodes(asList(expectedResult, episode));
        when(activity.getMedia()).thenReturn(episode);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);
        var result = playNextService.getNextEpisode();

        assertTrue(result.isPresent());
        assertEquals(expectedResult, result.get());
    }

    @Test
    void testOnPlayMedia_whenEpisodeIsLastEpisodeInShow_shouldNotUpdateNextEpisode() {
        var activity = mock(PlayMediaEvent.class);
        var episode1 = new Episode();
        var episode2 = new Episode();
        var show = Show.builder()
                .episodes(asList(episode1, episode2))
                .build();
        episode1.setEpisode(1);
        episode1.setShow(show);
        episode2.setEpisode(2);
        episode2.setShow(show);
        when(activity.getMedia()).thenReturn(episode2);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);
        var result = playNextService.getNextEpisode();

        assertTrue(result.isEmpty());
    }

    @Test
    void testOnPlayMedia_whenEpisodesAreNotInOrder_shouldSelectTheCorrectNextEpisode() {
        var activity = mock(PlayMediaEvent.class);
        var show = Show.builder().build();
        var episode1 = createEpisode(1, show);
        var episode2 = createEpisode(2, show);
        var episode3 = createEpisode(3, show);
        var episode4 = createEpisode(4, show);
        show.setEpisodes(asList(episode1, episode3, episode2, episode4));
        when(activity.getMedia()).thenReturn(episode2);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);
        var result = playNextService.getNextEpisode();

        assertTrue(result.isPresent(), "next episode should be available");
        assertEquals(episode3, result.get());
    }

    //region OnTimeChanged

    @Test
    void testOnTimeChanged_whenPlayNextIsDisabled_shouldNotUpdatePlayingIn() {
        var activity = mock(PlayMediaEvent.class);
        var episode = createEpisode();
        when(activity.getMedia()).thenReturn(episode);

        // update the next episode
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);
        playNextService.onPlayVideo(activity);

        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(false);
        playNextService.onDurationChanged(90);
        playNextService.onTimeChanged(70);
        var result = playNextService.getPlayingIn();

        assertEquals(PlayNextService.COUNTDOWN_FROM, result);
    }

    @Test
    void testOnTimeChanged_whenPlayNextIsEnabled_shouldUpdatePlayingInValue() {
        var activity = mock(PlayMediaEvent.class);
        var episode = createEpisode();
        var expectedResult = 20;
        when(activity.getMedia()).thenReturn(episode);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);
        playNextService.onDurationChanged(90000);
        playNextService.onTimeChanged(70000);
        var result = playNextService.getPlayingIn();

        assertEquals(expectedResult, result);
    }

    @Test
    void testOnTimeChanged_whenPlayNextIsEnabledAndRemainingTimeIsZero_shouldStopTheVideoPlayback() {
        var activity = mock(PlayMediaEvent.class);
        var episode = createEpisode();
        var videoLength = 90000;
        when(activity.getMedia()).thenReturn(episode);
        when(activity.getQuality()).thenReturn("480p");
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);
        playNextService.onDurationChanged(videoLength);
        playNextService.onTimeChanged(videoLength);

        verify(videoPlayerService).stop();
    }

    @Test
    void testOnTimeChanged_whenPlayNextIsEnabledAndRemainingTimeIsZero_shouldTriggerTheNextEpisodePlayback() {
        var activity = mock(PlayMediaEvent.class);
        var episode = createEpisode();
        var videoLength = 90000;
        when(activity.getMedia()).thenReturn(episode);
        when(activity.getQuality()).thenReturn("480p");
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayVideo(activity);
        playNextService.onDurationChanged(videoLength);
        playNextService.onTimeChanged(videoLength);

        verify(eventPublisher).publishEvent(isA(LoadMediaTorrentEvent.class));
    }

    //endregion

    //region PlayNextEpisodeNow

    @Test
    void testPlayNextEpisodeNow_whenNextEpisodeIsEmpty_shouldDoNothing() {
        playNextService.playNextEpisodeNow();

        verify(videoPlayerService, times(0)).stop();
        verify(eventPublisher, times(0)).publishEvent(isA(LoadMediaTorrentEvent.class));
    }

    //endregion

    //region Init

    @Test
    void testInit_whenInvoked_shouldAddVideoPlayerServiceListener() {
        playNextService.init();

        verify(videoPlayerService).addListener(isA(VideoPlayerListener.class));
    }

    @Test
    void testInit_whenVideoPlayerTimeIsChanged_shouldUpdateTime() {
        var listenerHolder = new AtomicReference<VideoPlayerListener>();
        var activity = mock(PlayMediaEvent.class);
        var show = Show.builder().build();
        var episode1 = createEpisode(1, show);
        var episode2 = createEpisode(2, show);
        show.setEpisodes(asList(episode1, episode2));
        var expectedResult = 20L;
        doAnswer(invocationOnMock -> {
            var listener = (VideoPlayerListener) invocationOnMock.getArgument(0);
            listenerHolder.set(listener);
            return null;
        }).when(videoPlayerService).addListener(isA(VideoPlayerListener.class));
        when(activity.getMedia()).thenReturn(episode1);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.init();
        playNextService.onPlayVideo(activity);

        var listener = listenerHolder.get();
        listener.onDurationChanged(100000L);
        listener.onTimeChanged(80000L);

        var result = playNextService.getPlayingIn();

        assertEquals(expectedResult, result);
    }

    //endregion

    private Episode createEpisode(int episode, Show show) {
        return Episode.builder()
                .episode(episode)
                .show(show)
                .build();
    }

    private Episode createEpisode() {
        var nextEpisode = new Episode();

        nextEpisode.setEpisode(2);
        nextEpisode.setTorrents(new HashMap<>());

        return createEpisode(nextEpisode);
    }

    private Episode createEpisode(Episode nextEpisode) {
        var episode = new Episode();
        var show = Show.builder()
                .episodes(asList(episode, nextEpisode))
                .build();
        var torrents = new HashMap<String, MediaTorrentInfo>();

        episode.setEpisode(1);
        episode.setTorrents(torrents);
        episode.setShow(show);
        nextEpisode.setTorrents(torrents);
        torrents.put("480p", mock(MediaTorrentInfo.class));
        torrents.put("720p", mock(MediaTorrentInfo.class));

        return episode;
    }
}

package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.ui.activities.ActivityManager;
import com.github.yoep.popcorn.ui.activities.LoadMediaTorrentActivity;
import com.github.yoep.popcorn.ui.activities.PlayMediaActivity;
import com.github.yoep.popcorn.ui.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.ui.settings.models.PlaybackSettings;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayNextServiceTest {
    @Mock
    private ActivityManager activityManager;
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
        when(settingsService.getSettings()).thenReturn(settings);
        when(settings.getPlaybackSettings()).thenReturn(playbackSettings);
    }

    @Test
    void testOnPlayMedia_whenMediaIsMovie_shouldNotUpdateNextEpisode() {
        var activity = mock(PlayMediaActivity.class);
        when(activity.getMedia()).thenReturn(mock(Movie.class));
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayMedia(activity);

        assertTrue(playNextService.getNextEpisode().isEmpty());
    }

    @Test
    void testOnPlayMedia_whenMediaIsEpisodeAndPlayNextIsDisabled_shouldNotUpdateNextEpisode() {
        var activity = mock(PlayMediaActivity.class);
        var episode = mock(Episode.class);
        lenient().when(activity.getMedia()).thenReturn(episode);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(false);

        playNextService.onPlayMedia(activity);

        assertTrue(playNextService.getNextEpisode().isEmpty());
    }

    @Test
    void testOnPlayMedia_whenMediaIsEpisodeAndPlayNextIsEnabled_shouldUpdateNextEpisode() {
        var activity = mock(PlayMediaActivity.class);
        var expectedResult = new Episode();
        var episode = createEpisode(expectedResult);
        when(activity.getMedia()).thenReturn(episode);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayMedia(activity);
        var result = playNextService.getNextEpisode();

        assertTrue(result.isPresent());
        assertEquals(expectedResult, result.get());
    }

    @Test
    void testOnPlayMedia_whenEpisodeIsLastEpisodeInShow_shouldNotUpdateNextEpisode() {
        var activity = mock(PlayMediaActivity.class);
        var episode1 = new Episode();
        var episode2 = new Episode();
        var show = new Show();
        episode1.setEpisode(1);
        episode2.setEpisode(2);
        show.setEpisodes(asList(episode1, episode2));
        when(activity.getMedia()).thenReturn(episode2);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayMedia(activity);
        var result = playNextService.getNextEpisode();

        assertTrue(result.isEmpty());
    }

    @Test
    void testOnTimeChanged_whenPlayNextIsDisabled_shouldNotUpdatePlayingIn() {
        var activity = mock(PlayMediaActivity.class);
        var episode = createEpisode();
        when(activity.getMedia()).thenReturn(episode);

        // update the next episode
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);
        playNextService.onPlayMedia(activity);

        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(false);
        playNextService.onDurationChanged(90);
        playNextService.onTimeChanged(70);
        var result = playNextService.getPlayingIn();

        assertEquals(PlayNextService.COUNTDOWN_FROM, result);
    }

    @Test
    void testOnTimeChanged_whenPlayNextIsEnabled_shouldUpdatePlayingInValue() {
        var activity = mock(PlayMediaActivity.class);
        var episode = createEpisode();
        var expectedResult = 20;
        when(activity.getMedia()).thenReturn(episode);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayMedia(activity);
        playNextService.onDurationChanged(90000);
        playNextService.onTimeChanged(70000);
        var result = playNextService.getPlayingIn();

        assertEquals(expectedResult, result);
    }

    @Test
    void testOnTimeChanged_whenPlayNextIsEnabledAndRemainingTimeIsZero_shouldStopTheVideoPlayback() {
        var activity = mock(PlayMediaActivity.class);
        var episode = createEpisode();
        var videoLength = 90000;
        when(activity.getMedia()).thenReturn(episode);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayMedia(activity);
        playNextService.onDurationChanged(videoLength);
        playNextService.onTimeChanged(videoLength);

        verify(videoPlayerService).close();
    }

    @Test
    void testOnTimeChanged_whenPlayNextIsEnabledAndRemainingTimeIsZero_shouldTriggerTheNextEpisodePlayback() {
        var activity = mock(PlayMediaActivity.class);
        var episode = createEpisode();
        var videoLength = 90000;
        when(activity.getMedia()).thenReturn(episode);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playNextService.onPlayMedia(activity);
        playNextService.onDurationChanged(videoLength);
        playNextService.onTimeChanged(videoLength);

        verify(activityManager).register(isA(LoadMediaTorrentActivity.class));
    }

    private Episode createEpisode() {
        var nextEpisode = new Episode();

        nextEpisode.setEpisode(2);

        return createEpisode(nextEpisode);
    }

    private Episode createEpisode(Episode nextEpisode) {
        var episode = new Episode();
        var show = new Show();

        episode.setEpisode(1);
        show.setEpisodes(asList(episode, nextEpisode));

        return episode;
    }
}

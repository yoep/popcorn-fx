package com.github.yoep.popcorn.ui.view.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.filters.models.Season;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.List;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ShowHelperServiceTest {
    @Mock
    private LocaleText localeText;
    @Mock
    private WatchedService watchedService;
    @InjectMocks
    private ShowHelperService showHelperService;

    @Nested
    class GetSeasonsTest {
        @Test
        void testGetSeasons_whenMediaIsNull_shouldThrowIllegalArgumentException() {
            assertThrows(IllegalArgumentException.class, () -> showHelperService.getSeasons(null), "media cannot be null");
        }

        @Test
        void testGetSeasons_whenMediaHas3Season_shouldReturn3Seasons() {
            var media = mock(Show.class);
            when(media.getNumberOfSeasons()).thenReturn(3);

            var result = showHelperService.getSeasons(media);

            assertNotNull(result);
            assertEquals(3, result.size());
        }

        @Test
        void testGetSeasons_whenInvoked_shouldRequestLocaleTextToDisplay() {
            var media = mock(Show.class);
            when(media.getNumberOfSeasons()).thenReturn(1);

            showHelperService.getSeasons(media);

            verify(localeText).get(DetailsMessage.SEASON, 1);
        }
    }

    @Nested
    class GetSeasonEpisodesTest {
        @Test
        void testGetSeasonEpisodes_whenMediaHasNoEpisodes_shouldReturnAnEmptyList() {
            var seasonNumber = 1;
            var season = new Season(seasonNumber, "season-display-text");
            var media = createShow(Collections.emptyList());

            var result = showHelperService.getSeasonEpisodes(season, media);

            assertNotNull(result);
            assertEquals(0, result.size());
        }

        @Test
        void testGetSeasonEpisodes_whenInvoked_shouldOnlyReturnsEpisodesFromSeason() {
            var seasonNumber = 1;
            var season = new Season(seasonNumber, "season-display-text");
            var episodeFromSeason = Episode.builder()
                    .season(seasonNumber)
                    .build();
            var episodeOfAnotherSeason = Episode.builder()
                    .season(2)
                    .build();
            var media = createShow(asList(episodeFromSeason, episodeOfAnotherSeason));

            var result = showHelperService.getSeasonEpisodes(season, media);

            assertNotNull(result);
            assertEquals(1, result.size());
            assertEquals(episodeFromSeason, result.get(0));
        }
    }

    @Nested
    class GetUnwatchedSeasonTest {
        @Test
        void testGetUnwatchedSeason_whenInvoked_shouldCheckIfTheEpisodeHasBeenWatched() {
            var seasonNumber = 1;
            var season = new Season(seasonNumber, "season");
            var seasons = Collections.singletonList(season);
            var episode = Episode.builder()
                    .season(seasonNumber)
                    .build();
            var show = createShow(Collections.singletonList(episode));

            showHelperService.getUnwatchedSeason(seasons, show);

            verify(watchedService).isWatched(episode);
        }

        @Test
        void testGetUnwatchedSeason_whenHasUnwatchedSeason_shouldReturnUnwatchedSeason() {
            var season1 = new Season(1, "season-1");
            var season2 = new Season(2, "season-2");
            var season3 = new Season(3, "season-3");
            var seasons = asList(season1, season2, season3);
            var episodeFromSeason2 = Episode.builder()
                    .season(2)
                    .build();
            var episodeFromSeason3 = Episode.builder()
                    .season(3)
                    .build();
            var show = createShow(asList(episodeFromSeason2, episodeFromSeason3));

            episodeFromSeason2.setWatched(false);
            episodeFromSeason3.setWatched(true);

            season1.setWatched(true);
            season2.setWatched(false);
            season3.setWatched(true);

            var result = showHelperService.getUnwatchedSeason(seasons, show);

            assertEquals(season2, result);
        }

        @Test
        void testGetUnwatchedSeason_whenAllSeasonAreWatched_shouldReturnLastSeason() {
            var season1 = new Season(1, "season-1");
            var season2 = new Season(2, "season-2");
            var seasons = asList(season1, season2);
            var show = createShow(Collections.emptyList());

            season1.setWatched(true);
            season2.setWatched(true);

            var result = showHelperService.getUnwatchedSeason(seasons, show);

            assertEquals(season2, result);
        }
    }

    private Show createShow(List<Episode> episodes) {
        return Show.builder()
                .episodes(episodes)
                .build();
    }
}

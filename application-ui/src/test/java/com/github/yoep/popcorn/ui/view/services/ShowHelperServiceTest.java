package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.media.Season;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.concurrent.CompletableFuture;

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
    private ShowHelperService ShowHelperService;

    @Nested
    class GetSeasonsTest {
        @Test
        void testGetSeasons_whenMediaIsNull_shouldThrowIllegalArgumentException() {
            assertThrows(NullPointerException.class, () -> ShowHelperService.getSeasons(null), "media cannot be null");
        }

        @Test
        void testGetSeasons_whenMediaHas3Season_shouldReturn3Seasons() {
            var media = new ShowDetails(Media.ShowDetails.newBuilder()
                    .setNumberOfSeasons(3)
                    .build());

            var result = ShowHelperService.getSeasons(media);

            assertNotNull(result);
            assertEquals(3, result.size());
        }

        @Test
        void testGetSeasons_whenInvoked_shouldRequestLocaleTextToDisplay() {
            var media = new ShowDetails(Media.ShowDetails.newBuilder()
                    .setNumberOfSeasons(1)
                    .build());

            ShowHelperService.getSeasons(media);

            verify(localeText).get(DetailsMessage.SEASON, 1);
        }
    }

    @Nested
    class GetSeasonEpisodesTest {
        @Test
        void testGetSeasonEpisodes_whenMediaHasNoEpisodes_shouldReturnAnEmptyList() {
            var seasonNumber = 1;
            var season = new Season(seasonNumber, "season-display-text");
            var media = mock(ShowDetails.class);
            when(media.getEpisodes()).thenReturn(Collections.emptyList());

            var result = ShowHelperService.getSeasonEpisodes(season, media);

            assertNotNull(result);
            assertEquals(0, result.size());
        }

        @Test
        void testGetSeasonEpisodes_whenInvoked_shouldOnlyReturnsEpisodesFromSeason() {
            var seasonNumber = 1;
            var season = new Season(seasonNumber, "season-display-text");
            var episodeFromSeason = new Episode(Media.Episode.newBuilder()
                    .setSeason(seasonNumber)
                    .build());
            var episodeOfAnotherSeason = new Episode(Media.Episode.newBuilder()
                    .setSeason(2)
                    .build());
            var media = new ShowDetails(Media.ShowDetails.newBuilder()
                    .addEpisodes(episodeFromSeason.proto())
                    .addEpisodes(episodeOfAnotherSeason.proto())
                    .build());

            var result = ShowHelperService.getSeasonEpisodes(season, media);

            assertNotNull(result);
            assertEquals(1, result.size());
            assertEquals(episodeFromSeason, result.getFirst());
        }
    }

    @Nested
    class GetUnwatchedSeasonTest {
        @Test
        void testGetUnwatchedSeason_whenInvoked_shouldCheckIfTheEpisodeHasBeenWatched() {
            var seasonNumber = 1;
            var season = new Season(seasonNumber, "season");
            var seasons = Collections.singletonList(season);
            var episode = new Episode(Media.Episode.newBuilder()
                    .setSeason(seasonNumber)
                    .build());
            var media = mock(ShowDetails.class);
            when(watchedService.isWatched(isA(com.github.yoep.popcorn.backend.media.Media.class))).thenReturn(CompletableFuture.completedFuture(false));
            when(media.getEpisodes()).thenReturn(Collections.singletonList(episode));

            ShowHelperService.getUnwatchedSeason(seasons, media).resultNow();

            verify(watchedService).isWatched(episode);
        }

        @Test
        void testGetUnwatchedSeason_whenHasUnwatchedSeason_shouldReturnUnwatchedSeason() {
            var season1 = new Season(1, "season-1");
            var season2 = new Season(2, "season-2");
            var season3 = new Season(3, "season-3");
            var seasons = asList(season1, season2, season3);
            var media = new ShowDetails(Media.ShowDetails.newBuilder()
                    .setNumberOfSeasons(3)
                    .addEpisodes(Media.Episode.newBuilder()
                            .setSeason(1)
                            .setEpisode(1)
                            .build())
                    .addEpisodes(Media.Episode.newBuilder()
                            .setSeason(1)
                            .setEpisode(2)
                            .build())
                    .addEpisodes(Media.Episode.newBuilder()
                            .setSeason(2)
                            .setEpisode(1)
                            .build())
                    .addEpisodes(Media.Episode.newBuilder()
                            .setSeason(3)
                            .setEpisode(1)
                            .build())
                    .build());
            when(watchedService.isWatched(isA(com.github.yoep.popcorn.backend.media.Media.class))).thenAnswer(invocations -> {
                var episode = invocations.getArgument(0, Episode.class);
                // return season 1 and 3 episodes as watched
                return CompletableFuture.completedFuture(episode.season() == 1 || episode.season() == 3);
            });

            var result = ShowHelperService.getUnwatchedSeason(seasons, media).resultNow();

            assertEquals(season2, result);
        }

        @Test
        void testGetUnwatchedSeason_whenAllSeasonAreWatched_shouldReturnLastSeason() {
            var season1 = new Season(1, "season-1");
            var season2 = new Season(2, "season-2");
            var seasons = asList(season1, season2);
            var media = new ShowDetails(Media.ShowDetails.newBuilder()
                    .addEpisodes(Media.Episode.newBuilder()
                            .setSeason(1)
                            .setEpisode(1)
                            .build())
                    .addEpisodes(Media.Episode.newBuilder()
                            .setSeason(2)
                            .setEpisode(1)
                            .build())
                    .addEpisodes(Media.Episode.newBuilder()
                            .setSeason(2)
                            .setEpisode(2)
                            .build())
                    .build());
            when(watchedService.isWatched(isA(com.github.yoep.popcorn.backend.media.Media.class))).thenReturn(CompletableFuture.completedFuture(true));

            var result = ShowHelperService.getUnwatchedSeason(seasons, media).resultNow();

            assertEquals(season2, result);
        }
    }
}

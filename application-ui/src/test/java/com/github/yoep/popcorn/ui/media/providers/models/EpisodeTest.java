package com.github.yoep.popcorn.ui.media.providers.models;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class EpisodeTest {
    @Test
    void testCompareTo_whenEpisodeSeasonIsBeforeCompareObject_shouldReturnNegativeOne() {
        var episode = Episode.builder()
                .season(2)
                .build();
        var episodeCompare = Episode.builder()
                .season(3)
                .build();
        var expectedResult = -1;

        var result = episode.compareTo(episodeCompare);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCompareTo_whenEpisodeSeasonIsAfterCompareObject_shouldReturnPlusOne() {
        var episode = Episode.builder()
                .season(6)
                .build();
        var episodeCompare = Episode.builder()
                .season(1)
                .build();
        var expectedResult = 1;

        var result = episode.compareTo(episodeCompare);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCompareTo_whenEpisodeIsBeforeCompareObject_shouldReturnNegativeOne() {
        var episode = Episode.builder()
                .season(3)
                .episode(2)
                .build();
        var episodeCompare = Episode.builder()
                .season(3)
                .episode(7)
                .build();
        var expectedResult = -1;

        var result = episode.compareTo(episodeCompare);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCompareTo_whenEpisodeIsAfterCompareObject_shouldReturnPlusOne() {
        var episode = Episode.builder()
                .season(1)
                .episode(4)
                .build();
        var episodeCompare = Episode.builder()
                .season(1)
                .episode(3)
                .build();
        var expectedResult = 1;

        var result = episode.compareTo(episodeCompare);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCompareTo_whenEpisodeIsSameAsCompareObject_shouldReturnZero() {
        var episode = Episode.builder()
                .season(1)
                .episode(4)
                .build();
        var episodeCompare = Episode.builder()
                .season(1)
                .episode(4)
                .build();
        var expectedResult = 0;

        var result = episode.compareTo(episodeCompare);

        assertEquals(expectedResult, result);
    }


}

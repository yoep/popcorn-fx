package com.github.yoep.popcorn.backend.media.favorites.sort;

import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.stream.Collectors;
import java.util.stream.Stream;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class FavoritesSortByWatchedStrategyTest {
    private FavoritesSortByWatchedStrategy strategy;

    @BeforeEach
    void setUp() {
        strategy = new FavoritesSortByWatchedStrategy();
    }

    @Test
    void testSupport_whenSortByIsWatched_shouldReturnTrue() {
        var result = strategy.support(new SortBy("watched", ""));

        assertTrue(result, "should support sort by watched");
    }

    @Test
    void testSort_whenMedia1IsWatched_shouldReturnMedia1AfterMedia2() {
        var media1 = Movie.builder().build();
        var media2 = Movie.builder().build();
        media1.setWatched(true);
        var stream = Stream.<Media>of(media1, media2);
        var expectedResult = asList(media2, media1);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }

    @Test
    void testSort_whenAllItemAreWatched_shouldNotSortTheStream() {
        var media1 = Movie.builder().build();
        var media2 = Movie.builder().build();
        var media3 = Movie.builder().build();
        media1.setWatched(true);
        media2.setWatched(true);
        media3.setWatched(true);
        var stream = Stream.<Media>of(media3, media1, media2);
        var expectedResult = asList(media3, media1, media2);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }

    @Test
    void testSort_whenMedia2IsMovie_shouldReturnMedia2BeforeMedia1() {
        var media1 = Show.builder().build();
        var media2 = Movie.builder().build();
        var stream = Stream.<Media>of(media1, media2);
        var expectedResult = asList(media2, media1);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }
}

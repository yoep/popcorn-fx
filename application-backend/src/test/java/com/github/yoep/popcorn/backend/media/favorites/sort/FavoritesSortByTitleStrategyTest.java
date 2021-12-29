package com.github.yoep.popcorn.backend.media.favorites.sort;

import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
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

class FavoritesSortByTitleStrategyTest {
    private FavoritesSortByTitleStrategy strategy;

    @BeforeEach
    void setUp() {
        strategy = new FavoritesSortByTitleStrategy();
    }

    @Test
    void testSupport_whenSortByIsTitle_shouldReturnTrue() {
        var result = strategy.support(new SortBy("title", ""));

        assertTrue(result, "should support sort by title");
    }

    @Test
    void testSort_whenMedia2TitleIsBeforeMedia1_shouldReturnMedia2First() {
        var media1 = Movie.builder()
                .title("beta")
                .build();
        var media2 = Movie.builder()
                .title("alpha")
                .build();
        var stream = Stream.<Media>of(media1, media2);
        var expectedResult = asList(media2, media1);

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

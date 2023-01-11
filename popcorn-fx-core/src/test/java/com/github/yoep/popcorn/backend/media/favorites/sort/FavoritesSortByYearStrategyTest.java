package com.github.yoep.popcorn.backend.media.favorites.sort;

import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.stream.Collectors;
import java.util.stream.Stream;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class FavoritesSortByYearStrategyTest {
    private FavoritesSortByYearStrategy strategy;

    @BeforeEach
    void setUp() {
        strategy = new FavoritesSortByYearStrategy();
    }

    @Test
    void testSupport_whenSortByIsYear_shouldReturnTrue() {
        var result = strategy.support(new SortBy("year", ""));

        assertTrue(result, "should support sort by year");
    }

    @Test
    void testSort_whenMedia1YearIsBefore_shouldReturnMedia1AfterMedia2() {
        var media1 = Movie.builder()
                .year("1990")
                .build();
        var media2 = Movie.builder()
                .year("2019")
                .build();
        var stream = Stream.<Media>of(media1, media2);
        var expectedResult = asList(media2, media1);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }

    @Test
    void testSort_whenMediaYearIsInvalid_shouldReturnItemAsLast() {
        var media1 = Movie.builder()
                .year("lorem")
                .build();
        var media2 = Movie.builder()
                .year("2000")
                .build();
        var stream = Stream.<Media>of(media1, media2);
        var expectedResult = asList(media2, media1);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }

    @Test
    void testSort_WhenInvoked_shouldOrderMediaItemsWithNewestFirst() {
        var media1 = Movie.builder()
                .year("1990")
                .build();
        var media2 = Movie.builder()
                .year("2019")
                .build();
        var media3 = Movie.builder()
                .year("2021")
                .build();
        var media4 = Movie.builder()
                .year("2009")
                .build();
        var stream = Stream.<Media>of(media1, media2, media3, media4);
        var expectedResult = asList(media3, media2, media4, media1);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }

    @Test
    void testSort_whenMedia2IsMovie_shouldReturnMedia2BeforeMedia1() {
        var media1 = new ShowOverview();
        var media2 = Movie.builder().build();
        var stream = Stream.<Media>of(media1, media2);
        var expectedResult = asList(media2, media1);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }
}

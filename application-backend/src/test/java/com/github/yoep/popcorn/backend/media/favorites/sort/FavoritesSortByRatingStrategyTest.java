package com.github.yoep.popcorn.backend.media.favorites.sort;

import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.stream.Collectors;
import java.util.stream.Stream;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class FavoritesSortByRatingStrategyTest {
    private FavoritesSortByRatingStrategy strategy;

    @BeforeEach
    void setUp() {
        strategy = new FavoritesSortByRatingStrategy();
    }

    @Test
    void testSupports_whenSortByIsRating_shouldReturnTrue() {
        var result = strategy.support(new SortBy("rating", ""));

        assertTrue(result, "should be true when sort by is rating");
    }

    @Test
    void testSort_whenMedia2RatingIsLower_shouldReturnMedia2AfterMedia1() {
        var media1 = createMedia(60);
        var media2 = createMedia(40);
        var stream = Stream.of(media2, media1);
        var expectedResult = asList(media1, media2);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }

    @Test
    void testSort_whenMedia2RatingIsHigher_shouldReturnMedia2BeforeMedia1() {
        var media1 = createMedia(60);
        var media2 = createMedia(80);
        var stream = Stream.of(media2, media1);
        var expectedResult = asList(media2, media1);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }

    @Test
    void testSort_whenMediaTypeIsDifferent_shouldReturnMoviesBeforeShows() {
        var show = Show.builder().build();
        var movie = Movie.builder().build();
        var stream = Stream.<Media>of(show, movie);
        var expectedResult = asList(movie, show);

        var result = strategy.sort(stream).collect(Collectors.toList());

        assertEquals(expectedResult, result);
    }

    private Media createMedia(int ratingPercentage) {
        return Movie.builder()
                .rating(Rating.builder()
                        .percentage(ratingPercentage)
                        .build())
                .build();
    }
}

package com.github.yoep.provider.anime.parsers.imdb;

import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import org.jsoup.Jsoup;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;

@ExtendWith(MockitoExtension.class)
class RatingParserTest {
    @Test
    void testExtractRating_whenRatingIsPresent_shouldReturnTheExpectedRating() {
        var document = Jsoup.parseBodyFragment("<div class=\"inline-block ratings-imdb-rating\">" +
                "<span class=\"global-sprite rating-star imdb-rating\"></span><strong>8.1</strong></div>");
        var expectedResult = Rating.builder()
                .percentage(81)
                .build();

        var result = RatingParser.extractRating(document.body());

        assertEquals(expectedResult, result);
    }
}
package com.github.yoep.provider.anime.parsers.imdb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class ImageParserTest {
    @Test
    void testExtractImage_whenSrcIsNull_shouldReturnNull() {
        var result = ImageParser.extractImage(null);

        assertNull(result);
    }

    @Test
    void testExtractImage_whenSrcDoesNotMatchExpectedPattern_shouldReturnNull() {
        var src = "https://www.imdb.com/title/tt0121955/?ref_=adv_li_tt";

        var result = ImageParser.extractImage(src);

        assertNull(result);
    }

    @Test
    void testExtractImage_whenSrcMatches_shouldReturnTheExpectedImage() {
        var src = "https://m.media-amazon.com/images/M/MV5BOGE2YWUzMDItNTg2Ny00NTUzLTlmZGYtNWMyNzVjMjQ3MThkXkEyXkFqcGdeQXVyNTA4NzY1MzY@._V1_UX67_CR0,0,67,98_AL_.jpg";
        var expectedResult = "MV5BOGE2YWUzMDItNTg2Ny00NTUzLTlmZGYtNWMyNzVjMjQ3MThkXkEyXkFqcGdeQXVyNTA4NzY1MzY";

        var result = ImageParser.extractImage(src);

        assertEquals(expectedResult, result);
    }
}
package com.github.yoep.provider.anime.parsers.imdb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class YearParserTest {
    @Test
    void testExtractStartYearFromSearch_whenEndYearIsPresent_shouldReturnStartYear() {
        var year = "(2013–2022)";
        var expectedResult = "2013";

        var result = YearParser.extractStartYearFromSearch(year);

        assertEquals(expectedResult, result);
    }

    @Test
    void testExtractStartYearFromSearch_whenEndYearIsNotPresent_shouldReturnStartYear() {
        var year = "(2019– )";
        var expectedResult = "2019";

        var result = YearParser.extractStartYearFromSearch(year);

        assertEquals(expectedResult, result);
    }

    @Test
    void testExtractStartYearFromSearch_whenNoYearIsPresent_shouldReturnNull() {
        var year = "";

        var result = YearParser.extractStartYearFromSearch(year);

        assertNull(result);
    }

    @Test
    void testExtractStartYearFromDetails_whenNoYearIsPresent_shouldReturnNull() {
        var year = "";

        var result = YearParser.extractStartYearFromDetails(year);

        assertNull(result);
    }

    @Test
    void testExtractStartYearFromDetails_whenYearIsIncomplete_shouldReturnStartYear() {
        var year = "1997-";
        var expectedResult = "1997";

        var result = YearParser.extractStartYearFromDetails(year);

        assertEquals(expectedResult, result);
    }

    @Test
    void testExtractStartYearFromDetails_whenYearIsComplete_shouldReturnStartYear() {
        var year = "2013-2022";
        var expectedResult = "2013";

        var result = YearParser.extractStartYearFromDetails(year);

        assertEquals(expectedResult, result);
    }
}
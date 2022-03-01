package com.github.yoep.provider.anime.parsers.imdb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class YearParserTest {
    @Test
    void testExtractStartYear_whenEndYearIsPresent_shouldReturnStartYear() {
        var year = "(2013–2022)";
        var expectedResult = "2013";

        var result = YearParser.extractStartYear(year);

        assertEquals(expectedResult, result);
    }

    @Test
    void testExtractStartYear_whenEndYearIsNotPresent_shouldReturnStartYear() {
        var year = "(2019– )";
        var expectedResult = "2019";

        var result = YearParser.extractStartYear(year);

        assertEquals(expectedResult, result);
    }

    @Test
    void testExtractStartYear_whenNoYearIsPresent_shouldReturnNull() {
        var year = "";

        var result = YearParser.extractStartYear(year);

        assertNull(result);
    }
}
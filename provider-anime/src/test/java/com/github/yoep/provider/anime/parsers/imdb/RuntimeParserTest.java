package com.github.yoep.provider.anime.parsers.imdb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class RuntimeParserTest {
    @Test
    void testExtractRuntime_whenRuntimeContainsMinutes_shouldReturnRuntime() {
        var expectedResult = 24;

        var result = RuntimeParser.extractRuntime(expectedResult + " min");

        assertEquals(expectedResult, result);
    }

    @Test
    void testExtractRuntime_whenRuntimeContainsM_shouldReturnRuntime() {
        var expectedResult = 48;

        var result = RuntimeParser.extractRuntime(expectedResult + "m");

        assertEquals(expectedResult, result);
    }
}
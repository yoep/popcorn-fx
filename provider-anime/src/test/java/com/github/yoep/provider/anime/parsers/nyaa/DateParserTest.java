package com.github.yoep.provider.anime.parsers.nyaa;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class DateParserTest {
    @Test
    void testConvertDateToYear_whenEpochIs2022_shouldReturn2022() {
        var epochValue = "Thu, 03 Feb 2022 15:48:54 -0000";
        var expectedResult = "2022";

        var result = DateParser.convertDateToYear(epochValue);

        assertEquals(expectedResult, result);
    }

    @Test
    void testConvertDateToYear_whenEpochIs2016_shouldReturn2016() {
        var epochValue = "Sun, 10 Jan 2016 04:57:00 -0000";
        var expectedResult = "2016";

        var result = DateParser.convertDateToYear(epochValue);

        assertEquals(expectedResult, result);
    }

    @Test
    void testConvertDateToYear_whenEpochIsInvalid_shouldReturnNull() {
        var epochValue = "qwerty";

        var result = DateParser.convertDateToYear(epochValue);

        assertNull(result);
    }
}
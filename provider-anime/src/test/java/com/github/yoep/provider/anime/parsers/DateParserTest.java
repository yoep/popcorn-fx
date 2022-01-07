package com.github.yoep.provider.anime.parsers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

class DateParserTest {
    @Test
    void testConvertDateToYear_whenEpochIs2018_shouldReturn2018() {
        var epochValue = "1518453282";
        var expectedResult = "2018";

        var result = DateParser.convertDateToYear(epochValue);

        assertEquals(expectedResult, result);
    }

    @Test
    void testConvertDateToYear_whenEpochIs2019_shouldReturn2019() {
        var epochValue = "1572299853";
        var expectedResult = "2019";

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
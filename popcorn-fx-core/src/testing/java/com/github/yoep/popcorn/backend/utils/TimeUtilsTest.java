package com.github.yoep.popcorn.backend.utils;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TimeUtilsTest {
    @Test
    void testFormat_whenTimeIsGiven_shouldReturnExpectedDisplayTime() {
        var time = 1050000;
        var expectedResult = "17:30";

        var result = TimeUtils.format(time);

        assertEquals(expectedResult, result);
    }
}
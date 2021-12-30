package com.github.yoep.popcorn.backend.settings.models;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class UIScaleTest {
    @Test
    void testToString_whenInvoked_shouldReturnTheValueAsString() {
        var value = 1.25f;
        var expectedValue = "125" + UIScale.APPENDIX;

        var result = new UIScale(value).toString();

        assertEquals(expectedValue, result);
    }
}

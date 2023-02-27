package com.github.yoep.video.javafx.conditions;

import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class OnMediaSupportedConditionTest {
    @Test
    void testMatches_shouldReturnTheMediaSupportResult() {
        var expectedResult = Platform.isSupported(ConditionalFeature.WEB);

        var result = OnMediaSupportedCondition.matches();

        assertEquals(expectedResult, result);
    }
}
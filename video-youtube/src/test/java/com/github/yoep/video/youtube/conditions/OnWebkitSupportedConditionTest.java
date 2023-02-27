package com.github.yoep.video.youtube.conditions;

import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class OnWebkitSupportedConditionTest {
    @Test
    void testMatches_shouldReturnWebResultOfPlatform() {
        var expectedResult = Platform.isSupported(ConditionalFeature.WEB);

        var result = OnWebkitSupportedCondition.matches();

        assertEquals(expectedResult, result);
    }
}
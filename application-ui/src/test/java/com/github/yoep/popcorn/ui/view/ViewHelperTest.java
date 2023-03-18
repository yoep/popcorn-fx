package com.github.yoep.popcorn.ui.view;

import javafx.util.Duration;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;

@ExtendWith(ApplicationExtension.class)
class ViewHelperTest {
    @Test
    void testInstantTooltip() {
        var result = ViewHelper.instantTooltip("lorem");

        assertEquals(Duration.ZERO, result.getShowDelay());
        assertEquals(Duration.INDEFINITE, result.getShowDuration());
        assertEquals(Duration.ZERO, result.getHideDelay());
    }
}
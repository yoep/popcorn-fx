package com.github.yoep.popcorn.ui.view.controls;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.junit.jupiter.api.Assertions.*;

@ExtendWith(ApplicationExtension.class)
class ProgressControlTest {
    @Test
    void testSetError() {
        var control = new ProgressControl();

        control.setError(true);
        assertTrue(control.getStyleClass().contains(ProgressControl.ERROR_STYLE_CLASS));

        control.setError(false);
        assertFalse(control.getStyleClass().contains(ProgressControl.ERROR_STYLE_CLASS));
    }

    @Test
    void testReset() {
        var control = new ProgressControl();
        control.setDuration(1000);
        control.setTime(500);
        control.setLoadProgress(0.5);

        control.reset();

        assertEquals(0, control.getDuration());
        assertEquals(0, control.getTime());
        assertEquals(0, control.getLoadProgress());
    }
}
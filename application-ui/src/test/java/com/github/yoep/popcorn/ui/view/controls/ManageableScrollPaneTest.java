package com.github.yoep.popcorn.ui.view.controls;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import static org.junit.jupiter.api.Assertions.assertNull;

@ExtendWith(ApplicationExtension.class)
class ManageableScrollPaneTest {
    private ManageableScrollPane pane;

    @BeforeEach
    void setUp() {
        this.pane = new ManageableScrollPane();
    }

    @Test
    void testDisableShortKeys() {
        pane.setShortKeysEnabled(false);
        WaitForAsyncUtils.waitForFxEvents();

        assertNull(pane.getEventDispatcher(), "expected the event dispatcher to have been removed");
    }
}
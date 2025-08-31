package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.info.ComponentInfo;
import com.github.yoep.popcorn.backend.info.ComponentState;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.time.Duration;
import java.util.ArrayList;
import java.util.Collections;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(ApplicationExtension.class)
class AboutDetailsTest {
    @Test
    void testInit_shouldAddStyleClass() {
        var component = new AboutDetails();

        assertTrue(component.getStyleClass().contains(AboutDetails.STYLE_CLASS), "expected the style class to have been present");
    }

    @Test
    void testSetItems_whenInvokedTwice_shouldPreventConcurrentModification() {
        var info1 = mock(ComponentInfo.class);
        var info2 = mock(ComponentInfo.class);
        when(info1.getState()).thenReturn(ComponentState.READY);
        when(info2.getState()).thenReturn(ComponentState.UNKNOWN);
        var threads = new ArrayList<Thread>();
        var component = new AboutDetails();

        threads.add(new Thread(() -> component.setItems(Collections.singletonList(info1))));
        threads.add(new Thread(() -> component.setItems(Collections.singletonList(info2))));

        threads.forEach(Thread::start);
        WaitForAsyncUtils.waitForFxEvents();

        assertTimeout(Duration.ofMillis(750), () -> component.getChildren().size() == 2, "expected 2 components");
    }
}
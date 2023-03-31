package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(ApplicationExtension.class)
class OverlayTest {
    @Test
    void testFocus() {
        var button = spy(new Button());
        var overlay = new Overlay(button);
        var parent = new AnchorPane(overlay);
        WaitForAsyncUtils.waitForFxEvents();
        var scene = new Scene(parent);
        WaitForAsyncUtils.waitForFxEvents();

        overlay.show();

        verify(button, timeout(250).atLeast(1)).requestFocus();
        assertEquals(1, GridPane.getColumnIndex(button), "expected column index 1");
        assertEquals(1, GridPane.getRowIndex(button), "expected row index 1");
    }

    @Test
    @Disabled
    void testHide() {
        var button = spy(new Button());
        var overlay = new Overlay(button);
        var parent = new AnchorPane();
        WaitForAsyncUtils.waitForFxEvents();
        var scene = new Scene(parent);
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitForFxEvents();
        parent.getChildren().add(overlay);
        overlay.hide();

        assertFalse(parent.getChildren().contains(overlay));
    }

    @Test
    void testAttachToParent_shouldScanUpwardsForParent() {
        var overlay = new Overlay();
        var inBetween = new Pane();
        var parent = new AnchorPane(inBetween);

        WaitForAsyncUtils.waitForFxEvents();
        inBetween.getChildren().add(overlay);
        WaitForAsyncUtils.waitForFxEvents();
        var scene = new Scene(parent);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(parent, overlay.attachedParent.get());
    }

    @Test
    void testStyleClass() {
        var overlay = new Overlay();

        WaitForAsyncUtils.waitForFxEvents();
        assertTrue(overlay.getStyleClass().contains(Overlay.STYLE_CLASS));
    }

    @Test
    void testStyleClassChilds() {
        var child1 = new Pane();
        var child2 = new Pane();

        var overlay = new Overlay(child1, child2);
        WaitForAsyncUtils.waitForFxEvents();
        assertTrue(child1.getStyleClass().contains(Overlay.CHILD_STYLE_CLASS));
        assertTrue(child2.getStyleClass().contains(Overlay.CHILD_STYLE_CLASS));

        overlay.getChildren().remove(child2);
        assertFalse(child2.getStyleClass().contains(Overlay.CHILD_STYLE_CLASS));
    }
}
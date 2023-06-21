package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.spy;

@ExtendWith(ApplicationExtension.class)
class OverlayTest {
    @Test
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

        assertTrue(overlay.isDisabled());
    }

    @Test
    void testAttachToSpecifiedID() throws TimeoutException {
        var overlay = new Overlay();
        var inBetween = new Pane(overlay);
        var expectedParent = new AnchorPane(inBetween);
        var root = new StackPane(expectedParent);
        expectedParent.setId(Overlay.DEFAULT_ATTACH_ID);

        WaitForAsyncUtils.waitForFxEvents();
        var scene = new Scene(root);
        overlay.requestLayout();
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> expectedParent.getChildren().contains(overlay));
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
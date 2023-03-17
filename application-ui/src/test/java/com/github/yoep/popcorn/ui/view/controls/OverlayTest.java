package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verify;

@ExtendWith(ApplicationExtension.class)
class OverlayTest {
    @Test
    void testFocus() {
        var parent = new Pane();
        var button = spy(new Button());
        var overlay = new Overlay(button);
        var scene = new Scene(parent);

        parent.getChildren().add(overlay);
        overlay.show();

        verify(button).requestFocus();
        assertEquals(1, GridPane.getColumnIndex(button), "expected column index 1");
        assertEquals(1, GridPane.getRowIndex(button), "expected row index 1");
    }

    @Test
    void testHide() {
        var button = spy(new Button());
        var overlay = new Overlay(button);
        var parent = new Pane();

        parent.getChildren().add(overlay);
        overlay.hide();

        assertFalse(parent.getChildren().contains(overlay));
    }

    @Test
    void testStyleClass() {
        var overlay = new Overlay();

        assertTrue(overlay.getStyleClass().contains(Overlay.STYLE_CLASS));
    }

    @Test
    void testStyleClassChilds() {
        var child1 = new Pane();
        var child2 = new Pane();

        var overlay = new Overlay(child1, child2);
        assertTrue(child1.getStyleClass().contains(Overlay.CHILD_STYLE_CLASS));
        assertTrue(child2.getStyleClass().contains(Overlay.CHILD_STYLE_CLASS));

        overlay.getChildren().remove(child2);
        assertFalse(child2.getStyleClass().contains(Overlay.CHILD_STYLE_CLASS));
    }
}
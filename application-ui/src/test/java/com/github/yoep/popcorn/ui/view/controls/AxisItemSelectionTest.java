package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(ApplicationExtension.class)
class AxisItemSelectionTest {
    @Test
    void testOrientationHorizontal() {
        var control = new AxisItemSelection<Genre>();
        control.setItems(new Genre("", ""));

        control.setOrientation(AxisItemSelection.Orientation.HORIZONTAL);

        assertTrue(control.getContent() instanceof HBox);
        assertEquals(1, ((Pane) control.getContent()).getChildren().size());
    }

    @Test
    void testOrientationVertical() {
        var control = new AxisItemSelection<Genre>();
        control.setItems(new Genre("", ""));

        control.setOrientation(AxisItemSelection.Orientation.VERTICAL);

        assertTrue(control.getContent() instanceof VBox);
        assertEquals(1, ((Pane) control.getContent()).getChildren().size());
    }

    @Test
    void testSetSelectedItem() {
        var item = new Genre("lorem", "ipsum");
        var control = new AxisItemSelection<Genre>();
        control.setItems(new Genre("ipsum", "dolor"), item, new Genre("estla", "coffee"));

        control.setSelectedItem(item);

        var children = ((Pane) control.getContent()).getChildren();
        assertFalse(children.get(0).getStyleClass().contains(AxisItemSelection.SELECTED_STYLE_CLASS));
        assertTrue(children.get(1).getStyleClass().contains(AxisItemSelection.SELECTED_STYLE_CLASS));
    }

    @Test
    void testRequestFocus() {
        var nodeHolder = new AtomicReference<Node>();
        var control = new AxisItemSelection<Genre>();
        var selectedItem = new Genre("lorem", "ipsum");
        var scene = new Scene(control);
        control.setItemFactory(item -> {
            var node = spy(new Button());
            if (item == selectedItem) {
                nodeHolder.set(node);
            }
            return node;
        });
        control.setItems(new Genre("ipsum", "dolor"), selectedItem, new Genre("sit", "coffee"));
        control.setSelectedItem(selectedItem);
        WaitForAsyncUtils.waitForFxEvents();

        control.requestFocus();
        WaitForAsyncUtils.waitForFxEvents();

        var node = nodeHolder.get();
        verify(node, timeout(200).atLeast(1)).requestFocus();
    }
}
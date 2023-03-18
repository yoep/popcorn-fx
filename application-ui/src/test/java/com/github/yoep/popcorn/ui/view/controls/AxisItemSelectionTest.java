package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.junit.jupiter.api.Assertions.*;

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
}
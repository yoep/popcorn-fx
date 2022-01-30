package com.github.yoep.popcorn.ui.view.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import javafx.geometry.VPos;
import javafx.scene.control.Label;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;

import java.util.List;

public class AboutDetails extends VBox {
    private static final String STYLE_CLASS = "about-details";
    private static final String GRID_STYLE_CLASS = "details-card";
    private static final String NAME_STYLE_CLASS = "name";

    public AboutDetails() {
        init();
    }

    //region Properties

    public void setItems(List<SimpleComponentDetails> items) {
        renderItems(items);
    }

    //endregion

    private void init() {
        this.getStyleClass().add(STYLE_CLASS);
    }

    private void renderItems(List<SimpleComponentDetails> items) {
        this.getChildren().clear();

        items.forEach(e -> {
            var grid = createGrid();
            var nameLabel = createNameNode(e);
            var descriptionLabel = new Label(e.getDescription().orElse(null));
            var stateIcon = createStateNode(e);

            grid.add(nameLabel, 0, 0);
            grid.add(descriptionLabel, 0, 1);
            grid.add(stateIcon, 1, 0, 1, 2);

            this.getChildren().add(grid);
        });
    }

    private static Label createNameNode(SimpleComponentDetails detail) {
        var node = new Label(detail.getName());
        node.getStyleClass().add(NAME_STYLE_CLASS);
        return node;
    }

    private static Icon createStateNode(SimpleComponentDetails detail) {
        var node = stateToIcon(detail.getState());
        node.setSizeFactor(2);
        GridPane.setValignment(node, VPos.TOP);
        return node;
    }

    private static GridPane createGrid() {
        var grid = new GridPane();
        var column1 = new ColumnConstraints();
        var column2 = new ColumnConstraints();

        column1.setHgrow(Priority.ALWAYS);

        grid.getStyleClass().add(GRID_STYLE_CLASS);
        grid.getColumnConstraints().addAll(column1, column2);

        return grid;
    }

    private static Icon stateToIcon(ComponentState state) {
        switch (state) {
            case UNKNOWN:
                return new Icon(Icon.QUESTION_CIRCLE_UNICODE);
            case READY:
                return new Icon(Icon.CHECK_CIRCLE_UNICODE);
            case ERROR:
            default:
                return new Icon(Icon.TIMES_CIRCLE_O_UNICODE);
        }
    }

}

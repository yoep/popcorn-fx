package com.github.yoep.popcorn.ui.view.controls;

import javafx.application.Platform;
import javafx.collections.ListChangeListener;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.*;
import lombok.extern.slf4j.Slf4j;

import java.util.Optional;

@Slf4j
public class Overlay extends GridPane {
    static final String STYLE_CLASS = "overlay";
    static final String CHILD_STYLE_CLASS = "overlay-content";

    private Pane parent;
    private Node lastKnownFocusNode;

    public Overlay() {
        init();
    }

    public Overlay(Node... children) {
        init();
        getChildren().addAll(children);
    }

    /**
     * Verify if the overlay is currently being shown.
     *
     * @return Returns true when shown, else false.
     */
    public boolean isShowing() {
        return Optional.ofNullable(parent)
                .map(Pane::getChildren)
                .map(e -> e.contains(this))
                .orElse(false);
    }

    public void show() {
        if (!isShowing()) {
            var children = parent.getChildren();
            children.add(children.size(), this);
        }

        doInternalFocusRequest();
    }

    public void hide() {
        var children = parent.getChildren();
        children.remove(this);

        if (lastKnownFocusNode != null) {
            lastKnownFocusNode.requestFocus();
        }
    }

    private void init() {
        getStyleClass().add(STYLE_CLASS);
        getColumnConstraints().add(0, resizingColumn());
        getColumnConstraints().add(1, new ColumnConstraints());
        getColumnConstraints().add(2, resizingColumn());
        getRowConstraints().add(0, resizingRow());
        getRowConstraints().add(1, new RowConstraints());
        getRowConstraints().add(2, resizingRow());

        AnchorPane.setTopAnchor(this, 0d);
        AnchorPane.setRightAnchor(this, 0d);
        AnchorPane.setBottomAnchor(this, 0d);
        AnchorPane.setLeftAnchor(this, 0d);

        setOnKeyPressed(this::onKeyPressed);
        setOnMouseClicked(this::onMouseClicked);

        initializeListeners();
    }

    private void initializeListeners() {
        getChildren().addListener((ListChangeListener<? super Node>) change -> {
            while (change.next()) {
                if (change.wasAdded()) {
                    for (Node child : change.getAddedSubList()) {
                        GridPane.setColumnIndex(child, 1);
                        GridPane.setRowIndex(child, 1);
                        child.getStyleClass().add(CHILD_STYLE_CLASS);
                    }
                }
                if (change.wasRemoved()) {
                    for (Node child : change.getRemoved()) {
                        child.getStyleClass().removeIf(e -> e.contains(CHILD_STYLE_CLASS));
                    }
                }
            }
        });
        parentProperty().addListener((observable, oldValue, newValue) -> {
            if (this.parent == null && newValue instanceof Pane pane) {
                this.parent = pane;
                Platform.runLater(this::hide);
            }
        });
        focusWithinProperty().addListener((observable, oldValue, newValue) -> {
            if (!newValue && isShowing()) {
                doInternalFocusRequest();
            }
        });
    }

    private void onMouseClicked(MouseEvent event) {
        var x = event.getSceneX();
        var y = event.getSceneY();

        if (getChildren().stream().noneMatch(e -> e.getBoundsInParent().contains(x, y))) {
            event.consume();
            hide();
        }
    }

    private void onKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.ESCAPE) {
            event.consume();
            hide();
        }
    }

    private void doInternalFocusRequest() {
        lastKnownFocusNode = Optional.ofNullable(getScene())
                .map(Scene::getFocusOwner)
                .orElse(null);
        for (Node child : getChildren()) {
            var node = findFocusableNode(child);
            if (node != null) {
                node.requestFocus();
                break;
            }
        }
    }

    private static Node findFocusableNode(Node node) {
        if (node instanceof Region region) {
            for (Node child : region.getChildrenUnmodifiable()) {
                var focusableNode = findFocusableNode(child);
                if (focusableNode != null) {
                    return focusableNode;
                }
            }
        }

        if (node.isFocusTraversable()) {
            return node;
        }

        return null;
    }

    private static ColumnConstraints resizingColumn() {
        var resizingColumn = new ColumnConstraints();
        resizingColumn.setMaxWidth(Double.MAX_VALUE);
        resizingColumn.setHgrow(Priority.ALWAYS);
        return resizingColumn;
    }

    private static RowConstraints resizingRow() {
        var resizingRow = new RowConstraints();
        resizingRow.setMaxHeight(Double.MAX_VALUE);
        resizingRow.setVgrow(Priority.ALWAYS);
        return resizingRow;
    }
}

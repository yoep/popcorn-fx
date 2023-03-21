package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.ListChangeListener;
import javafx.scene.Node;
import javafx.scene.Parent;
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

    final ObjectProperty<Node> forNode = new SimpleObjectProperty<>(this, "for");
    final ObjectProperty<AnchorPane> attachedParent = new SimpleObjectProperty<>(this, "attachedParent");
    final BooleanProperty shown = new SimpleBooleanProperty(this, "shown");

    Node lastKnownFocusNode;

    public Overlay() {
        init();
    }

    public Overlay(Node... children) {
        init();
        getChildren().addAll(children);
    }

    //region Properties

    public Node getFor() {
        return forNode.get();
    }

    public ObjectProperty<Node> forProperty() {
        return forNode;
    }

    public void setFor(Node forNode) {
        this.forNode.set(forNode);
    }

    public boolean isShown() {
        return shown.get();
    }

    public BooleanProperty shownProperty() {
        return shown;
    }

    public void setShown(boolean shown) {
        this.shown.set(shown);
    }

    //endregion

    public void show() {
        if (!isShown()) {
            if (attachedParent.get() == null) {
                attachToParent(getParent());
            }

            var children = attachedParent.get().getChildren();
            children.add(children.size(), this);
            setShown(true);
        }

        doInternalFocusRequest();
    }

    public void hide() {
        var attachedParent = this.attachedParent.get();
        if (attachedParent == null)
            return;

        var children = attachedParent.getChildren();
        children.remove(this);
        setShown(false);

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
        getChildren().addListener((ListChangeListener<? super Node>) Overlay::onChildrenChanged);
        attachedParent.addListener((observable, oldValue, newValue) -> ((Pane) getParent()).getChildren().remove(this));
        sceneProperty().addListener((observable, oldValue, newValue) -> updateParentIfNeeded());
        parentProperty().addListener((observable, oldValue, newValue) -> updateParentIfNeeded());
        focusWithinProperty().addListener((observable, oldValue, newValue) -> {
            if (!newValue && isShown()) {
                doInternalFocusRequest();
            }
        });
        forNode.addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                newValue.setOnMouseClicked(event -> {
                    event.consume();
                    show();
                });
                newValue.setOnKeyPressed(event -> {
                    if (event.getCode() == KeyCode.ENTER) {
                        event.consume();
                        show();
                    }
                });
            }
            if (oldValue != null) {
                oldValue.setOnMouseClicked(null);
                oldValue.setOnKeyPressed(null);
            }
        });
    }

    private void updateParentIfNeeded() {
        if (attachedParent.get() == null) {
            attachToParent(getParent());
        }
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
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.ESCAPE || event.getCode() == KeyCode.UNDEFINED) {
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

    private void attachToParent(Parent parent) {
        if (parent instanceof AnchorPane pane) {
            attachedParent.set(pane);
            log.trace("Overlay has been attached to {}", pane);
        } else if (parent != null) {
            attachToParent(parent.getParent());
        }
    }

    private static void onChildrenChanged(ListChangeListener.Change<? extends Node> change) {
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
        resizingColumn.setMinWidth(45d);
        resizingColumn.setMaxWidth(Double.MAX_VALUE);
        resizingColumn.setHgrow(Priority.ALWAYS);
        return resizingColumn;
    }

    private static RowConstraints resizingRow() {
        var resizingRow = new RowConstraints();
        resizingRow.setMinHeight(25d);
        resizingRow.setMaxHeight(Double.MAX_VALUE);
        resizingRow.setVgrow(Priority.ALWAYS);
        return resizingRow;
    }
}

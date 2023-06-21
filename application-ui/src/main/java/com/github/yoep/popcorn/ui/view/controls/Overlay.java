package com.github.yoep.popcorn.ui.view.controls;

import javafx.application.Platform;
import javafx.beans.property.*;
import javafx.collections.ListChangeListener;
import javafx.geometry.HPos;
import javafx.geometry.VPos;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Region;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;
import java.util.Optional;

@Slf4j
public class Overlay extends Pane {
    static final String STYLE_CLASS = "overlay";
    static final String CHILD_STYLE_CLASS = "overlay-content";
    static final String DEFAULT_ATTACH_ID = "root";

    final ObjectProperty<Node> forNode = new SimpleObjectProperty<>(this, "for");
    final StringProperty attachTo = new SimpleStringProperty(this, "attachTo", DEFAULT_ATTACH_ID);
    final BooleanProperty shown = new SimpleBooleanProperty(this, "shown");

    Node lastKnownFocusNode;
    boolean attached;

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

    public String getAttachTo() {
        return attachTo.get();
    }

    public StringProperty attachToProperty() {
        return attachTo;
    }

    public void setAttachTo(String attachTo) {
        this.attachTo.set(attachTo);
    }

    //endregion

    public synchronized void show() {
        setViewOrder(-999);
        updateState(false);
        setShown(true);
        doInternalFocusRequest();
    }

    public synchronized void hide() {
        setViewOrder(999);
        updateState(true);
        setShown(false);

        if (lastKnownFocusNode != null) {
            lastKnownFocusNode.requestFocus();
        }
    }

    @Override
    public void requestLayout() {
        if (!attached && getScene() != null) {
            log.trace("Searching for node ID {} to attach to", getAttachTo());
            doAttaching(getScene().getRoot().getChildrenUnmodifiable());
            hide();
        }

        super.requestLayout();
    }

    @Override
    protected void layoutChildren() {
        var children = getManagedChildren();
        var width = getWidth();
        var height = getHeight();
        var insets = getInsets();
        var usableWidth = width - insets.getLeft() - insets.getRight();
        var usableHeight = height - insets.getTop() - insets.getBottom();

        for (Node child : children) {
            layoutInArea(child, 0, 0, usableWidth, usableHeight, 0, getInsets(), false, false, HPos.CENTER, VPos.CENTER);
        }
    }

    private void init() {
        getStyleClass().add(STYLE_CLASS);
        AnchorPane.setTopAnchor(this, 0d);
        AnchorPane.setLeftAnchor(this, 0d);

        setOnKeyPressed(this::onKeyPressed);
        setOnMouseClicked(this::onMouseClicked);

        initializeListeners();
    }

    private void initializeListeners() {
        getChildren().addListener((ListChangeListener<? super Node>) Overlay::onChildrenChanged);
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

    private void doAttaching(List<Node> nodes) {
        for (Node node : nodes) {
            if (node instanceof Pane pane) {
                if (Objects.equals(node.getId(), getAttachTo())) {
                    if (!pane.getChildren().contains(this)) {
                        pane.getChildren().add(pane.getChildren().size(), this);
                    }

                    prefWidthProperty().bind(pane.widthProperty());
                    prefHeightProperty().bind(pane.heightProperty());
                    attached = true;
                    return;
                }

                doAttaching(pane.getChildren());
            }
        }
    }

    private void onMouseClicked(MouseEvent event) {
        var x = event.getSceneX();
        var y = event.getSceneY();

        if (getChildren().stream().noneMatch(e -> e.getBoundsInParent().contains(x, y))) {
            event.consume();
            hide();
        }

        if (event.getButton() == MouseButton.BACK) {
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
        doInternalFocusRequest(0);
    }

    private void doInternalFocusRequest(int attempt) {
        Platform.runLater(() -> {
            for (Node child : getChildren()) {
                var node = findFocusableNode(child);
                if (node != null) {
                    node.requestFocus();

                    if (node.isFocused())
                        return;
                }
            }

            if (attempt < 15) {
                new Thread(() -> {
                    try {
                        Thread.sleep(100);
                    } catch (InterruptedException e) {
                        log.warn(e.getMessage(), e);
                    }

                    doInternalFocusRequest(attempt + 1);
                }, "Overlay.AttachParent").start();
            }
        });
    }

    private void updateState(boolean disabled) {
        setDisable(disabled);
        setVisible(!disabled);
        getChildren().forEach(e -> e.setDisable(disabled));

        if (getParent() == null)
            return;

        for (Node node : getParent().getChildrenUnmodifiable()) {
            if (node != this) {
                node.setDisable(!disabled);
            }
        }
    }

    private static void onChildrenChanged(ListChangeListener.Change<? extends Node> change) {
        while (change.next()) {
            if (change.wasAdded()) {
                for (Node child : change.getAddedSubList()) {
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
}

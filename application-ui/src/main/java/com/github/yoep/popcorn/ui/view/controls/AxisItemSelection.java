package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.MapChangeListener;
import javafx.collections.ObservableMap;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.input.KeyCode;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;

import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * The axis item selection allows the selection of a certain item on the X- or Y-axis.
 *
 * @param <T> The item that will be displayed for this selection.
 */
public class AxisItemSelection<T> extends ManageableScrollPane {
    static final String STYLE_CLASS = "axis";
    static final String CONTENT_STYLE_CLASS = "content";
    static final String SELECTED_STYLE_CLASS = "selected";

    /**
     * The available items within the selection.
     */
    private final ObservableMap<T, Node> items = FXCollections.observableHashMap();
    /**
     * The selected item.
     */
    private final ObjectProperty<T> selectedItem = new SimpleObjectProperty<>();
    private final ObjectProperty<Orientation> orientation = new SimpleObjectProperty<>(Orientation.VERTICAL);
    private final ObjectProperty<ItemFactory<T>> factory = new SimpleObjectProperty<>(item -> new Button(item.toString()));

    private Pane content;

    public AxisItemSelection() {
        super();
        initContent();
    }

    //region Properties

    public Orientation getOrientation() {
        return orientation.get();
    }

    public ReadOnlyObjectProperty<Orientation> orientationProperty() {
        return orientation;
    }

    public void setOrientation(Orientation orientation) {
        this.orientation.set(orientation);
    }

    public T getSelectedItem() {
        return selectedItem.get();
    }

    public ReadOnlyObjectProperty<T> selectedItemProperty() {
        return selectedItem;
    }

    public void setSelectedItem(T selectedItem) {
        setSelectedItem(selectedItem, false);
    }

    public void setSelectedItem(T selectedItem, boolean focus) {
        this.selectedItem.set(selectedItem);
        scrollTo(selectedItem, focus);
    }

    public ItemFactory<T> getFactory() {
        return factory.get();
    }

    public void setFactory(ItemFactory<T> factory) {
        this.factory.set(factory);
    }

    //endregion

    public List<T> getItems() {
        return items.keySet().stream().toList();
    }

    public void add(T item) {
        items.put(item, createNewItem(item));
    }

    public void addAll(T... items) {
        for (T item : items) {
            add(item);
        }
    }

    public void setItems(T... items) {
        clear();
        addAll(items);
    }

    public void scrollTo(T item) {
        scrollTo(item, false);
    }

    public void scrollTo(T item, boolean focus) {
        Optional.ofNullable(items.get(item))
                .ifPresent(e -> {
                    var contentLocalBounds = getContent().getBoundsInLocal();
                    var x = e.getBoundsInParent().getMaxX();
                    var y = e.getBoundsInParent().getMaxY();

                    setVvalue(y / contentLocalBounds.getWidth());
                    setHvalue(x / contentLocalBounds.getHeight());

                    if (focus)
                        e.requestFocus();
                });
    }

    private void handleItemSelected(Node node) {
        for (Map.Entry<T, Node> entry : items.entrySet()) {
            if (entry.getValue() == node) {
                selectedItem.set(entry.getKey());
                return;
            }
        }
    }

    private void clear() {
        this.selectedItem.set(null);
        this.items.clear();
    }

    private Node createNewItem(T item) {
        var node = getFactory().createNode(item);

        node.setFocusTraversable(true);
        node.setOnMouseClicked(event -> {
            event.consume();
            handleItemSelected((Node) event.getSource());
        });
        node.setOnKeyPressed(event -> {
            if (event.getCode() == KeyCode.ENTER) {
                event.consume();
                handleItemSelected((Node) event.getSource());
            }
        });

        content.getChildren().add(node);
        return node;
    }

    private void initContent() {
        this.setHbarPolicy(ScrollBarPolicy.NEVER);
        this.setVbarPolicy(ScrollBarPolicy.NEVER);
        this.getStyleClass().add(STYLE_CLASS);

        items.addListener((MapChangeListener<? super T, ? super Node>) change -> {
            if (change.wasRemoved()) {
                content.getChildren().remove(change.getValueRemoved());
            }
        });
        selectedItem.addListener((observable, oldValue, newValue) -> {
            Optional.ofNullable(newValue)
                    .map(items::get)
                    .map(Node::getStyleClass)
                    .ifPresent(e -> e.add(SELECTED_STYLE_CLASS));
            Optional.ofNullable(oldValue)
                    .map(items::get)
                    .map(Node::getStyleClass)
                    .ifPresent(e -> e.removeIf(x -> x.equals(SELECTED_STYLE_CLASS)));
        });
        factory.addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                content.getChildren().clear();
                for (Map.Entry<T, Node> entry : items.entrySet()) {
                    var node = createNewItem(entry.getKey());
                    entry.setValue(node);
                }
            }
        });

        orientationProperty().addListener((observable, oldValue, newValue) -> updateOrientation());
        updateOrientation();
    }

    private void updateOrientation() {
        var children = Optional.ofNullable(content)
                .map(Pane::getChildren)
                .map(e -> e.toArray(new Node[0]))
                .orElse(new Node[0]);

        this.content = getOrientation() == Orientation.VERTICAL ? new VBox(children) : new HBox(children);
        this.content.getStyleClass().add(CONTENT_STYLE_CLASS);
        this.setContent(content);

        if (getOrientation() == Orientation.HORIZONTAL) {
            setPrefHeight(content.getHeight());
            prefHeightProperty().bind(content.heightProperty());
            prefWidthProperty().unbind();
        } else {
            setPrefWidth(content.getWidth());
            prefWidthProperty().bind(content.widthProperty());
            prefHeightProperty().unbind();
        }
    }

    public enum Orientation {
        VERTICAL,
        HORIZONTAL
    }
}

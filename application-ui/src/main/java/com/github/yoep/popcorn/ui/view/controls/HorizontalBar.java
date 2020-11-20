package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.ListChangeListener;
import javafx.collections.ObservableList;
import javafx.scene.Node;
import javafx.scene.layout.HBox;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;

@Slf4j
public class HorizontalBar<T> extends HBox {
    public static final String SELECTED_ITEM_PROPERTY = "selectedItem";
    public static final String ITEM_FACTORY_PROPERTY = "itemFactory";
    private static final String STYLE_CLASS = "horizontal-bar";
    private static final String ACTIVE_STYLE_CLASS = "active";
    private static final String ITEM_STYLE_CLASS = "item";

    private final ObservableList<T> items = FXCollections.observableArrayList();
    private final ObjectProperty<T> selectedItem = new SimpleObjectProperty<>(this, SELECTED_ITEM_PROPERTY);
    private final ObjectProperty<ItemFactory<T>> itemFactory = new SimpleObjectProperty<>(this, ITEM_FACTORY_PROPERTY);
    private final Map<T, Node> itemNodes = new ConcurrentHashMap<>();

    public HorizontalBar() {
        initialize();
    }

    //region Properties

    public ObservableList<T> getItems() {
        return items;
    }

    public ItemFactory<T> getItemFactory() {
        return itemFactory.get();
    }

    public ObjectProperty<ItemFactory<T>> itemFactoryProperty() {
        return itemFactory;
    }

    public void setItemFactory(ItemFactory<T> itemFactory) {
        this.itemFactory.set(itemFactory);
    }

    public T getSelectedItem() {
        return selectedItem.get();
    }

    public ObjectProperty<T> selectedItemProperty() {
        return selectedItem;
    }

    public void setSelectedItem(T item) {
        this.selectedItem.set(item);
    }

    //endregion

    //region Methods

    /**
     * Select the given item within this horizontal bar.
     *
     * @param item The item that needs to be selected.
     */
    public void select(T item) {
        if (items.contains(item)) {
            selectedItem.set(item);
        } else {
            log.warn("Item {} is not part of this HorizontalBar instance", item);
        }
    }

    //endregion

    //region Functions

    private void initialize() {
        initializeSelectedItem();
        initializeItems();

        getStyleClass().add(STYLE_CLASS);
    }

    private void initializeSelectedItem() {
        selectedItem.addListener((observable, oldValue, newValue) -> {
            Optional.ofNullable(oldValue)
                    .map(itemNodes::get)
                    .ifPresent(e -> e.getStyleClass().remove(ACTIVE_STYLE_CLASS));
            Optional.ofNullable(newValue)
                    .map(itemNodes::get)
                    .ifPresent(e -> {
                        e.requestFocus();
                        e.getStyleClass().add(ACTIVE_STYLE_CLASS);
                    });
        });
    }

    private void initializeItems() {
        items.addListener((ListChangeListener<? super T>) change -> {
            while (change.next()) {
                if (change.wasAdded()) {
                    addItems(change.getAddedSubList());
                } else if (change.wasRemoved()) {
                    removeItems(change.getRemoved());
                }
            }
        });
    }

    private void addItems(List<? extends T> items) {
        var factory = itemFactory.get();

        if (factory == null)
            throw new IllegalStateException("Item factory has not been initialized");

        for (T item : items) {
            var node = factory.createNode(item);

            node.setFocusTraversable(true);
            node.getStyleClass().add(ITEM_STYLE_CLASS);

            itemNodes.put(item, node);
            getChildren().add(node);
        }
    }

    private void removeItems(List<? extends T> items) {
        for (T item : items) {
            if (itemNodes.containsKey(item)) {
                getChildren().remove(itemNodes.get(item));
                itemNodes.remove(item);
            }
        }
    }

    //endregion
}

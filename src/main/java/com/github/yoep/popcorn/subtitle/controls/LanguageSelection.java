package com.github.yoep.popcorn.subtitle.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.beans.value.ChangeListener;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.scene.image.ImageView;
import javafx.scene.layout.HBox;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

public class LanguageSelection<T> extends HBox {
    private static final String IMAGE_VIEW_STYLE_CLASS = "flag";
    private static final String ARROW_STYLE_CLASS = "arrow";

    private final ImageView imageView = new ImageView();
    private final Icon arrow = new Icon(Icon.CARET_UP_UNICODE);

    private final ObjectProperty<ObservableList<T>> items = new SimpleObjectProperty<>(this, "items");
    private final List<ChangeListener<T>> listeners = new ArrayList<>();

    public LanguageSelection() {
        this.items.set(FXCollections.observableArrayList());

        init();
    }

    /**
     * Get the items from this node.
     *
     * @return Returns the items of this node.
     */
    public ObservableList<T> getItems() {
        return items.get();
    }

    /**
     * The items property of this node.
     *
     * @return Returns the items property.
     */
    public ObjectProperty<ObservableList<T>> itemsProperty() {
        return items;
    }

    /**
     * Register the listener to this node.
     *
     * @param listener The listener to register.
     */
    public void addListener(ChangeListener<T> listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    /**
     * Remove the listener from this node.
     *
     * @param listener The listener to remove.
     */
    public void removeListener(ChangeListener<T> listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    /**
     * Expand the language selection.
     */
    public void show() {

    }

    public void select(int index) {

    }

    public void select(T item) {

    }

    private void init() {
        initializeFlag();
        initializeArrow();
    }

    private void initializeFlag() {
        imageView.getStyleClass().add(IMAGE_VIEW_STYLE_CLASS);
        this.getChildren().add(imageView);
    }

    private void initializeArrow() {
        arrow.getStyleClass().add(ARROW_STYLE_CLASS);
        this.getChildren().add(arrow);
    }

    private void switchSelectedItem(T item) {

    }
}

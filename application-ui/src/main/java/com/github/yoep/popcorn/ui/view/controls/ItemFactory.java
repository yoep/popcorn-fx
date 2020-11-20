package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.Node;

/**
 * Defines a factory which converts items into a JavaFX graphics {@link Node}.
 *
 * @param <T> The item which will be converted.
 */
public interface ItemFactory<T> {
    /**
     * Create a {@link Node} for the given item.
     *
     * @param item The item to convert into a JavaFX node.
     * @return Returns a JavaFX node.
     */
    Node createNode(T item);
}

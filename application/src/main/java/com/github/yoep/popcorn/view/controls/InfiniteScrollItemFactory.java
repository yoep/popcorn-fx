package com.github.yoep.popcorn.view.controls;

import javafx.scene.Node;

import java.util.List;
import java.util.concurrent.CompletableFuture;

public interface InfiniteScrollItemFactory<T> {
    /**
     * Load the items for the given page.
     *
     * @param page The page to load (zero-index).
     * @return Returns the list of items for the page.
     */
    CompletableFuture<List<T>> loadPage(int page);

    /**
     * Create the cell for the given item.
     *
     * @param item The item to create the cell of.
     * @return Returns the visual node for the item.
     */
    Node createCell(T item);
}

package com.github.yoep.popcorn.ui.view.controls;

import javafx.application.Platform;
import javafx.beans.property.IntegerProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleIntegerProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.ListChangeListener;
import javafx.collections.ObservableList;
import javafx.scene.Node;
import javafx.scene.control.ScrollPane;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.Pane;
import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.Objects;
import java.util.stream.Collectors;

@Slf4j
public class InfiniteScrollPane<T> extends ManageableScrollPane {
    public static final String PAGE_PROPERTY = "page";
    public static final String ITEM_FACTORY_PROPERTY = "itemFactory";
    public static final String LOADER_FACTORY_PROPERTY = "loaderFactory";
    public static final String CONTENT_STYLE_CLASS = "infinite-content";

    private static final int SCROLLBAR_THRESHOLD = 90;

    private final FlowPane itemsPane = new FlowPane();
    /**
     * Specifies the current page of the infinite scroll pane.
     */
    private final IntegerProperty page = new SimpleIntegerProperty(this, PAGE_PROPERTY, 0);
    /**
     * The item factory for this {@link InfiniteScrollPane}.
     * This factory is invoked for each new item that is being added to the {@link InfiniteScrollPane}.
     */
    private final ObjectProperty<InfiniteScrollItemFactory<T>> itemFactory = new SimpleObjectProperty<>(this, ITEM_FACTORY_PROPERTY);
    /**
     * The loader factory for this {@link InfiniteScrollPane}.
     * This factory is invoked each time a new page is being loaded and will display the "loader node" while the page items are being retrieved.
     */
    private final ObjectProperty<LoaderFactory> loaderFactory = new SimpleObjectProperty<>(this, LOADER_FACTORY_PROPERTY);
    /**
     * The items of this {@link InfiniteScrollPane}.
     */
    private final ObservableList<ItemWrapper> items = FXCollections.observableArrayList();

    private final Object loaderLock = new Object();
    private final Object contentUpdaterLock = new Object();

    private Pane loader;
    private boolean updating;
    private boolean endOfItems;
    private double lastKnownScrollPosition;
    private Node lastFocusedItem;
    private Thread contentUpdater;

    //region Constructors

    public InfiniteScrollPane() {
        super();
        initScrollPane();
    }

    //endregion

    //region Properties

    /**
     * Get the current page of the infinite scroll pane.
     *
     * @return Returns the current page of the scroll pane.
     */
    public int getPage() {
        return page.get();
    }

    /**
     * Get the page property of the infinite scroll pane.
     *
     * @return Returns the page property.
     */
    public IntegerProperty pageProperty() {
        return page;
    }

    /**
     * Set the page for the infinite scroll pane.
     *
     * @param page The new page.
     */
    public void setPage(int page) {
        log.trace("Updating page to {}", page);
        this.page.set(page);
    }

    /**
     * Get the item factory of the infinite scroll pane.
     *
     * @return Returns the item factory.
     */
    public InfiniteScrollItemFactory<T> getItemFactory() {
        return itemFactory.get();
    }

    /**
     * Get the item factory property of this infinite scroll pane.
     *
     * @return Returns the item factory property.
     */
    public ObjectProperty<InfiniteScrollItemFactory<T>> itemFactoryProperty() {
        return itemFactory;
    }

    /**
     * Set the item factory for this infinite scroll pane.
     *
     * @param itemFactory The new item factory for this infinite scroll pane.
     */
    public void setItemFactory(InfiniteScrollItemFactory<T> itemFactory) {
        Assert.notNull(itemFactory, "itemFactory cannot be null");
        this.itemFactory.set(itemFactory);
    }

    /**
     * Get the loader factory of the infinite scroll pane.
     *
     * @return Returns the loader factory.
     */
    public LoaderFactory getLoaderFactory() {
        return loaderFactory.get();
    }

    /**
     * Get the load factory of the infinite scroll pane.
     *
     * @return Returns the loader factory property.
     */
    public ObjectProperty<LoaderFactory> loaderFactoryProperty() {
        return loaderFactory;
    }

    /**
     * Set the new loader factory for the infinite scroll pane.
     *
     * @param loaderFactory The new loader factory.
     */
    public void setLoaderFactory(LoaderFactory loaderFactory) {
        this.loaderFactory.set(loaderFactory);
    }

    //endregion

    //region Methods

    /**
     * Check if the {@link InfiniteScrollPane} contains the given item.
     *
     * @param item The item to check for.
     * @return Returns true if it contains the item, else false.
     */
    public boolean contains(T item) {
        return items.stream()
                .anyMatch(e -> e.getItem() == item);
    }

    /**
     * Reset the infinite scroll pane.
     * This will reset the page to "0" and remove all items from this control.
     */
    public void reset() {
        log.trace("Resetting the infinite scroll pane content");

        // finish up the current update
        finished();

        // cancel the content updater if it is still alive
        cancelContentUpdater();

        // reset the page
        endOfItems = false;
        updating = true;
        lastKnownScrollPosition = 0;
        lastFocusedItem = null;
        synchronized (contentUpdaterLock) {
            items.clear();
            setPage(0);
        }
        updating = false;

        runOnFx(() -> {
            synchronized (contentUpdaterLock) {
                itemsPane.getChildren().clear();
            }
        });
    }

    /**
     * Load a new page into the infinite scroll pane.
     */
    public void loadNewPage() {
        if (!updating && !endOfItems)
            increasePage();
    }

    //endregion

    //region Functions

    private void initScrollPane() {
        initializeScrollBars();
        initializeContent();
        initializeListeners();
        initializeSceneListener();
    }

    private void initializeScrollBars() {
        this.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        this.setVbarPolicy(ScrollBarPolicy.AS_NEEDED);

        this.vvalueProperty().addListener((observable, oldValue, newValue) -> onScroll());
    }

    private void initializeContent() {
        this.setFitToWidth(true);
        this.setContent(itemsPane);

        itemsPane.getStyleClass().add(CONTENT_STYLE_CLASS);
    }

    private void initializeListeners() {
        pageProperty().addListener((observable, oldValue, newValue) -> onPageChanged());
        items.addListener(this::onItemsChanged);
    }

    private void initializeSceneListener() {
        sceneProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null && lastKnownScrollPosition > 0) {
                resumeLastScrollPosition();
            } else if (newValue == null) {
                rememberLastFocusedItem();
            }
        });
    }

    private void onScroll() {
        var currentScroll = this.getVvalue();
        var maxScroll = this.getVmax();
        var vPercentage = (currentScroll / maxScroll) * 100;

        // store the current scroll position
        if (currentScroll != 0)
            this.lastKnownScrollPosition = currentScroll;

        if (vPercentage > SCROLLBAR_THRESHOLD && !updating)
            loadNewPage();
    }

    private void resumeLastScrollPosition() {
        Platform.runLater(() -> {
            this.setVvalue(lastKnownScrollPosition);

            // check if the focus needs the be traversed to a specific item
            // based on the last known focus before the infinite scroll pane was remove from the view tree
            if (lastFocusedItem != null) {
                lastFocusedItem.requestFocus();
            }
        });
    }

    private void rememberLastFocusedItem() {
        items.stream()
                .map(ItemWrapper::getGraphicsNode)
                .filter(Node::isFocusTraversable)
                .filter(Node::isFocused)
                .findFirst()
                .ifPresent(e -> lastFocusedItem = e);
    }

    private void addNodes(Node... nodes) {
        // remove the loader if it still present
        removeLoaderItem();

        // add the new node to the list
        runOnFx(() -> itemsPane.getChildren().addAll(nodes));
    }

    private void removeNodes(Node... nodes) {
        runOnFx(() -> itemsPane.getChildren().removeAll(nodes));
    }

    private void finished() {
        updating = false;

        removeLoaderItem();
    }

    private void removeLoaderItem() {
        synchronized (loaderLock) {
            if (loader != null) {
                runOnFx(() -> {
                    itemsPane.getChildren().remove(loader);
                    loader = null;
                });
            }
        }
    }

    private void increasePage() {
        setPage(getPage() + 1);
    }

    private void onPageChanged() {
        // check if the content is already being updated
        // if so, ignore this page change
        if (updating)
            return;

        updating = true;

        var loaderFactory = getLoaderFactory();
        var itemFactory = getItemFactory();

        // add the loader node if the load factory has been set
        synchronized (loaderLock) {
            if (loaderFactory != null && loader == null) {
                runOnFx(() -> {
                    loader = loaderFactory.get();
                    itemsPane.getChildren().add(loader);
                });
            }
        }

        // retrieve all the new items for the new page
        itemFactory
                .loadPage(getPage())
                .thenAccept(pageItems -> {
                    synchronized (contentUpdaterLock) {
                        log.trace("Creating new content updater");
                        contentUpdater = new Thread(() -> {
                            // when the total page items is smaller than 10
                            // assume that we have reached the end
                            if (pageItems.size() < 10)
                                endOfItems = true;

                            var nodes = pageItems.stream()
                                    .map(this::createWrapperForItem)
                                    .filter(Objects::nonNull)
                                    .collect(Collectors.toList());

                            this.items.addAll(nodes);

                            startAutomaticPageLoaderWatcher();
                        }, "InfiniteScrollPane-contentUpdater");

                        log.trace("Starting new content updater thread");
                        contentUpdater.start();
                    }
                });
    }

    private ItemWrapper createWrapperForItem(T item) {
        var itemFactory = getItemFactory();

        try {
            var graphicsNode = itemFactory.createCell(item);

            return new ItemWrapper(item, graphicsNode);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }

        return null;
    }

    private void onItemsChanged(ListChangeListener.Change<? extends ItemWrapper> change) {
        while (change.next()) {
            if (change.wasAdded()) {
                var nodes = change.getAddedSubList().stream()
                        .map(ItemWrapper::getGraphicsNode)
                        .toArray(Node[]::new);

                addNodes(nodes);
            } else if (change.wasRemoved()) {
                var nodes = change.getRemoved().stream()
                        .map(ItemWrapper::getGraphicsNode)
                        .toArray(Node[]::new);

                removeNodes(nodes);
            }
        }
    }

    private void cancelContentUpdater() {
        synchronized (contentUpdaterLock) {
            if (contentUpdater != null && contentUpdater.isAlive()) {
                log.trace("Cancelling the current content updater");
                contentUpdater.interrupt();
            }
        }
    }

    private void startAutomaticPageLoaderWatcher() {
        var lastChange = System.currentTimeMillis();
        var originHeight = itemsPane.getHeight();

        // wait till the content pane is stable
        // as long as the content pane is changing in height, we keep waiting
        while (System.currentTimeMillis() - lastChange < 500) {
            var currentHeight = itemsPane.getHeight();

            if (originHeight != currentHeight) {
                originHeight = currentHeight;
                lastChange = System.currentTimeMillis();
            }
        }

        // once the content pane is stable
        // we verify if we need to automatically load the next page
        Platform.runLater(() -> {
            var scrollPaneHeight = this.getHeight();
            var minContentHeight = scrollPaneHeight * 1.25;
            var actualContentHeight = itemsPane.getHeight();

            // check if enough items were loaded for the scrollbar to be scrollable
            // if not, load an additional page
            if (!endOfItems && actualContentHeight < minContentHeight && getPage() <= 3) {
                updating = false;
                increasePage();
            } else {
                // remove the loader
                finished();
            }

            // check if we need to traverse the focus to an item in the list
            traverseFocusToItems();
        });
    }

    private void traverseFocusToItems() {
        // check if any items is focusable and already has the focus
        // if so, we're not going to traverse the focus
        var focusAlreadyTraversed = items.stream()
                .map(ItemWrapper::getGraphicsNode)
                .filter(Node::isFocusTraversable)
                .anyMatch(Node::isFocused);

        if (focusAlreadyTraversed)
            return;

        // if we can traverse the focus and none of the items has the focus
        // we move
        items.stream()
                .map(ItemWrapper::getGraphicsNode)
                .filter(Node::isFocusTraversable)
                .findFirst()
                .ifPresent(Node::requestFocus);
    }

    private void runOnFx(Runnable runnable) {
        if (Platform.isFxApplicationThread()) {
            executeRunnable(runnable);
        } else {
            Platform.runLater(() -> executeRunnable(runnable));
        }
    }

    private void executeRunnable(Runnable runnable) {
        try {
            runnable.run();
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    //endregion

    @Getter
    @AllArgsConstructor
    private class ItemWrapper {
        private final T item;
        private final Node graphicsNode;
    }
}

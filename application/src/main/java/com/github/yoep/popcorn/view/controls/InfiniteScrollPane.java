package com.github.yoep.popcorn.view.controls;

import javafx.application.Platform;
import javafx.beans.property.IntegerProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleIntegerProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.MapChangeListener;
import javafx.collections.ObservableMap;
import javafx.scene.Node;
import javafx.scene.control.ScrollPane;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

@Slf4j
public class InfiniteScrollPane<T> extends ScrollPane {
    public static final String PAGE_PROPERTY = "page";
    public static final String ITEM_FACTORY_PROPERTY = "itemFactory";
    public static final String LOADER_FACTORY_PROPERTY = "loaderFactory";

    private static final int SCROLLBAR_THRESHOLD = 90;

    private final FlowPane itemsPane = new FlowPane();
    private final IntegerProperty page = new SimpleIntegerProperty(this, PAGE_PROPERTY, 0);
    private final ObjectProperty<InfiniteScrollItemFactory<T>> itemFactory = new SimpleObjectProperty<>(this, ITEM_FACTORY_PROPERTY);
    private final ObjectProperty<LoaderFactory> loaderFactory = new SimpleObjectProperty<>(this, LOADER_FACTORY_PROPERTY);
    private final ObservableMap<T, Node> items = FXCollections.observableHashMap();
    private final Object loaderLock = new Object();
    private final Object contentUpdaterLock = new Object();

    private Pane loader;
    private boolean updating;
    private boolean endOfItems;
    private Thread contentUpdater;

    //region Constructors

    public InfiniteScrollPane() {
        init();
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

    /**
     * Get the items of the infinite scroll pane.
     *
     * @return Returns the items of this instance.
     */
    public ObservableMap<T, Node> getItems() {
        return items;
    }

    //endregion

    //region Methods

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
        synchronized (contentUpdaterLock) {
            getItems().clear();
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
        if (!updating)
            increasePage();
    }

    //endregion

    //region Functions

    private void init() {
        initializeScrollBars();
        initializeContent();
        initializeListeners();
    }

    private void initializeScrollBars() {
        this.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        this.setVbarPolicy(ScrollBarPolicy.AS_NEEDED);

        this.vvalueProperty().addListener((observable, oldValue, newValue) -> onScroll());
    }

    private void initializeContent() {
        this.setFocusTraversable(true);
        this.setFitToWidth(true);
        this.setContent(itemsPane);
    }

    private void initializeListeners() {
        pageProperty().addListener((observable, oldValue, newValue) -> onPageChanged());
        getItems().addListener(this::onItemsChanged);
    }

    private void onScroll() {
        double vPercentage = (this.getVvalue() / this.getVmax()) * 100;

        if (vPercentage > SCROLLBAR_THRESHOLD && !updating)
            loadNewPage();
    }

    private void addNode(Node node) {
        // remove the loader if it still present
        removeLoaderItem();

        // add the new node to the list
        runOnFx(() -> itemsPane.getChildren().add(node));
    }

    private void removeNode(Node node) {
        runOnFx(() -> itemsPane.getChildren().remove(node));
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
                .thenAccept(items -> {
                    synchronized (contentUpdaterLock) {
                        log.trace("Creating new content updater");
                        contentUpdater = new Thread(() -> {
                            if (items.size() == 0)
                                endOfItems = true;

                            for (T item : items) {
                                runOnFx(() -> {
                                    // safely iterate over the new item and add it to the infinite scroll pane
                                    try {
                                        this.items.put(item, itemFactory.createCell(item));
                                    } catch (Exception ex) {
                                        log.error(ex.getMessage(), ex);
                                    }
                                });
                            }

                            runOnFx(() -> {
                                // check if enough items were loaded for the scrollbar to be scrollable
                                if (!endOfItems && itemsPane.getHeight() < (this.getHeight() * 1.5) && getPage() < 5) {
                                    // load an additional page
                                    updating = false;
                                    increasePage();
                                } else {
                                    // remove the loader
                                    finished();
                                }
                            });
                        }, "InfiniteScrollPane-contentUpdater");

                        log.trace("Starting new content updater thread");
                        contentUpdater.start();
                    }
                });
    }

    private void onItemsChanged(MapChangeListener.Change<? extends T, ? extends Node> change) {
        if (change.wasAdded()) {
            addNode(change.getValueAdded());
        } else {
            removeNode(change.getValueRemoved());
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
}

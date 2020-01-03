package com.github.yoep.popcorn.controls;

import javafx.application.Platform;
import javafx.beans.property.IntegerProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleIntegerProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.ObservableList;
import javafx.scene.Node;
import javafx.scene.control.ScrollPane;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

@Slf4j
public class InfiniteScrollPane extends ScrollPane {
    public static final String PAGE_PROPERTY = "page";
    public static final String LOADER_FACTORY_PROPERTY = "loaderFactory";

    private static final int SCROLLBAR_THRESHOLD = 90;

    private final FlowPane itemsPane = new FlowPane();
    private final IntegerProperty page = new SimpleIntegerProperty(this, PAGE_PROPERTY, 0);
    private final ObjectProperty<LoaderFactory> loaderFactory = new SimpleObjectProperty<>(this, LOADER_FACTORY_PROPERTY);

    private Pane loader;
    private boolean updating;

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
        this.page.set(page);
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
     * Add a new item to this infinite scroll pane.
     *
     * @param item The item to add to this control.
     */
    public void addItem(Node item) {
        Assert.notNull(item, "item cannot be null");

        Platform.runLater(() -> {
            ObservableList<Node> children = itemsPane.getChildren();
            int loaderIndex = children.indexOf(loader);

            if (loaderIndex != -1) {
                // add item before the loader item
                children.add(loaderIndex, item);
            } else {
                children.add(item);
            }
        });
    }

    /**
     * Set the loading state to finished.
     */
    public void finished() {
        updating = false;

        if (loader != null) {
            Platform.runLater(() -> {
                itemsPane.getChildren().remove(loader);
                loader = null;
            });
        }
    }

    /**
     * Reset the infinite scroll pane.
     * This will reset the page to "0" and remove all items from this control.
     */
    public void reset() {
        setPage(0);
        Platform.runLater(() -> itemsPane.getChildren().clear());
        updating = false;
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
        this.setVbarPolicy(ScrollPane.ScrollBarPolicy.ALWAYS);

        this.vvalueProperty().addListener((observable, oldValue, newValue) -> onScroll());
    }

    private void initializeContent() {
        this.setFocusTraversable(true);
        this.setFitToWidth(true);
        this.setContent(itemsPane);
    }

    private void initializeListeners() {
        pageProperty().addListener((observable, oldValue, newValue) -> onPageChanged(oldValue.intValue(), newValue.intValue()));
    }

    private void onScroll() {
        double vPercentage = (this.getVvalue() / this.getVmax()) * 100;

        if (vPercentage > SCROLLBAR_THRESHOLD && !updating) {
            increasePage();
        }
    }

    private void increasePage() {
        setPage(getPage() + 1);
    }

    private void onPageChanged(int oldValue, int newValue) {
        // check if the content is already being updated
        // if so, ignore this page change
        if (updating)
            return;

        updating = true;

        LoaderFactory loaderFactory = getLoaderFactory();

        if (loaderFactory != null && loader == null) {
            Platform.runLater(() -> {
                loader = loaderFactory.get();
                itemsPane.getChildren().add(loader);
            });
        }
    }

    //endregion
}

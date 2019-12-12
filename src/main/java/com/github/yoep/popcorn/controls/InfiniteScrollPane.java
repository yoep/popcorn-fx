package com.github.yoep.popcorn.controls;

import javafx.application.Platform;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.ProgressIndicator;
import javafx.scene.control.ScrollPane;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.VBox;
import javafx.scene.text.Text;
import lombok.extern.slf4j.Slf4j;

import java.util.ArrayList;
import java.util.List;

@Slf4j
public class InfiniteScrollPane extends ScrollPane {
    private static final int SCROLLBAR_THRESHOLD = 97;

    private final ProgressIndicator progressIndicator = new ProgressIndicator();
    private final Text noSearchResultsFound = new Text();
    private final FlowPane itemsPane = new FlowPane();
    private final VBox contentPane = new VBox(itemsPane, progressIndicator, noSearchResultsFound);
    private final List<PageListener> pageListeners = new ArrayList<>();

    private int page;
    private boolean updating;

    //region Constructors

    public InfiniteScrollPane() {
        initializeScrollBars();
        initializeContent();
    }

    //endregion

    //region Methods

    /**
     * Add a new {@link PageListener} to this infinite scroll pane.
     *
     * @param listener The listener to add.
     */
    public void addListener(PageListener listener) {
        synchronized (pageListeners) {
            pageListeners.add(listener);
        }
    }

    /**
     * Add a new item to this infinite scroll pane.
     *
     * @param item The item to add to this control.
     */
    public void addItem(Node item) {
        Platform.runLater(() -> itemsPane.getChildren().add(item));
    }

    /**
     * Reset the infinite scroll pane.
     * This will reset the page to "0" and remove all items from this control.
     */
    public void reset() {
        page = 0;
        Platform.runLater(() -> itemsPane.getChildren().clear());
    }

    /**
     * Load a new page into the infinite scroll pane.
     * This will invoke all the {@link PageListener}'s that are currently registered.
     */
    public void loadNewPage() {
        if (!updating)
            loadNextPage();
    }

    //endregion

    //region Functions

    private void initializeScrollBars() {
        this.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        this.setVbarPolicy(ScrollPane.ScrollBarPolicy.ALWAYS);

        this.vvalueProperty().addListener((observable, oldValue, newValue) -> onScroll());
    }

    private void initializeContent() {
        this.setFocusTraversable(true);
        this.setFitToWidth(true);
        this.contentPane.setAlignment(Pos.CENTER);
        this.setContent(contentPane);
    }

    private void onScroll() {
        double vPercentage = (this.getVvalue() / this.getVmax()) * 100;

        if (vPercentage > SCROLLBAR_THRESHOLD && !updating) {
            loadNextPage();
        }
    }

    private void loadNextPage() {
        updating = true;
        progressIndicator.setVisible(true);

        int previousPage = page;
        page = page + 1;

        synchronized (pageListeners) {
            pageListeners.forEach(e -> e.onChange(previousPage, page));
        }

        progressIndicator.setVisible(false);
        updating = false;
    }

    //endregion
}

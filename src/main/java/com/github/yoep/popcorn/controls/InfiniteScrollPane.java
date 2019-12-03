package com.github.yoep.popcorn.controls;

import javafx.geometry.Pos;
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

    //region Constructors

    public InfiniteScrollPane() {
        initializeScrollBars();
        initializeContent();
    }

    //endregion

    public FlowPane getItemsPane() {
        return itemsPane;
    }

    public void addListener(PageListener listener) {
        synchronized (pageListeners) {
            pageListeners.add(listener);
        }
    }

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

        if (vPercentage > SCROLLBAR_THRESHOLD) {
            loadNextPage();
        }
    }

    private void loadNextPage() {
        progressIndicator.setVisible(true);
        int previousPage = page;

        page = page + 1;

        synchronized (pageListeners) {
            pageListeners.forEach(e -> e.onChange(previousPage, page));
        }

        progressIndicator.setVisible(false);
    }
}

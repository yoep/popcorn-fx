package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controls.InfiniteScrollPane;
import javafx.fxml.FXML;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressIndicator;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;

@Slf4j
public abstract class AbstractListSectionController {
    protected final List<ProviderService<? extends Media>> providerServices;
    protected final ViewLoader viewLoader;
    protected final LocaleText localeText;
    protected final EventPublisher eventPublisher;

    protected Category category;
    protected Genre genre;
    protected SortBy sortBy;
    protected String search;
    /**
     * Indicates how many pages have failed to load for the current retrieval
     */
    protected int numberOfPageFailures;

    protected CompletableFuture<? extends List<? extends Media>> currentLoadRequest;

    @FXML
    protected InfiniteScrollPane<Media> scrollPane;
    @FXML
    protected Pane failedPane;
    @FXML
    protected Label failedText;
    @FXML
    protected Pane overlay;

    protected ProgressIndicator loadingIndicator;

    //region Constructors

    protected AbstractListSectionController(List<ProviderService<? extends Media>> providerServices,
                                            ViewLoader viewLoader,
                                            LocaleText localeText,
                                            EventPublisher eventPublisher) {
        this.eventPublisher = eventPublisher;
        Objects.requireNonNull(providerServices, "providerServices cannot be null");
        Objects.requireNonNull(viewLoader, "viewLoader cannot be null");
        Objects.requireNonNull(localeText, "localeText cannot be null");
        this.providerServices = providerServices;
        this.viewLoader = viewLoader;
        this.localeText = localeText;
    }

    //endregion

    //region Initializable

    protected void initializeFailedPane() {
        failedPane.setVisible(false);
    }

    protected void initializeOverlay() {
        overlay.setVisible(false);
    }

    //endregion

    //region Functions

    /**
     * Create a new FX node for the given item.
     *
     * @param item The item to create a node for.
     * @return Returns the node for the given item.
     */
    protected abstract Node createItemNode(Media item);

    protected void invokeNewPageLoad() {
        if (scrollPane != null && category != null && genre != null && sortBy != null)
            scrollPane.loadNewPage();
    }

    protected void reset() {
        if (currentLoadRequest != null)
            currentLoadRequest.cancel(true);

        numberOfPageFailures = 0;
        scrollPane.reset();
    }

    protected void showOverlay() {
        // check if the overlay is already shown
        // if so, ignore the action
        if (overlay.isVisible())
            return;

        if (loadingIndicator == null)
            loadingIndicator = new ProgressIndicator();

        overlay.getChildren().clear();
        overlay.getChildren().add(loadingIndicator);
        overlay.setVisible(true);
    }

    //endregion
}

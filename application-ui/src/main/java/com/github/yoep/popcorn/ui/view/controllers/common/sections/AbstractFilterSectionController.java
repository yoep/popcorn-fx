package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.StartScreen;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.Node;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.util.Assert;

import java.util.Optional;

@Slf4j
public abstract class AbstractFilterSectionController {
    protected static final String ACTIVE_STYLE_CLASS = "active";

    protected final ApplicationEventPublisher eventPublisher;
    protected final ApplicationConfig settingsService;

    @FXML
    protected Node moviesCategory;
    @FXML
    protected Node seriesCategory;
    @FXML
    protected Node animeCategory;
    @FXML
    protected Node favoritesCategory;

    protected Node lastKnownSelectedCategory;

    private boolean startScreenInitialized;

    //region Constructors

    protected AbstractFilterSectionController(ApplicationEventPublisher eventPublisher, ApplicationConfig settingsService) {
        Assert.notNull(eventPublisher, "eventPublisher cannot be null");
        Assert.notNull(settingsService, "settingsService cannot be null");
        this.eventPublisher = eventPublisher;
        this.settingsService = settingsService;
    }

    //endregion

    //region Functions

    /**
     * Clear the current search filter.
     */
    protected abstract void clearSearch();

    /**
     * Update the genre options based on the given category.
     *
     * @param category The category to base the genres on.
     */
    protected abstract void updateGenres(Category category);

    /**
     * Update the sort by options based on the given category.
     *
     * @param category The category to base the sort by on.
     */
    protected abstract void updateSortBy(Category category);

    /**
     * Initialize the scene listener for this {@link AbstractFilterSectionController}.
     *
     * @param rootNode The root node to attach the scene listener to.
     */
    protected void initializeSceneListener(Node rootNode) {
        Assert.notNull(rootNode, "rootNode cannot be null");
        rootNode.sceneProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(() -> {
            if (!startScreenInitialized) {
                var startScreen = getUiSettings().getStartScreen();

                initializeStartScreen(startScreen);
                startScreenInitialized = true;
            }
        }));
    }

    /**
     * Switch the active category to the given item.
     *
     * @param item The category item to switch to.
     */
    protected void switchCategory(Node item) {
        Assert.notNull(item, "item cannot be null");
        Category category;
        this.lastKnownSelectedCategory = item;

        removeAllActiveStates();
        activateItem(item);

        // set the active category based on the node
        // if the node is unknown as a category
        // ignore the switch action and log a warning
        if (item == moviesCategory) {
            category = Category.MOVIES;
        } else if (item == seriesCategory) {
            category = Category.SERIES;
        }  else if (item == animeCategory) {
            category = Category.ANIME;
        } else if (item == favoritesCategory) {
            category = Category.FAVORITES;
        } else {
            log.warn("Failed to switch category, unknown category item \"{}\"", item);
            return;
        }

        // invoke the change activity first before changing the genre & sort by
        log.trace("Category is being changed to \"{}\"", category);
        eventPublisher.publishEvent(new CategoryChangedEvent(this, category));

        // clear the current search
        log.trace("Clearing the current search filter");
        clearSearch();

        // set the category specific genres and sort by filters
        updateGenres(category);
        updateSortBy(category);
    }

    /**
     * Active the given node by applying the {@link #ACTIVE_STYLE_CLASS} style.
     *
     * @param item The node to apply the active style on.
     */
    protected void activateItem(Node item) {
        Optional.ofNullable(item)
                .map(Node::getStyleClass)
                .ifPresent(e -> e.add(ACTIVE_STYLE_CLASS));
    }

    /**
     * Remove the active states from all category nodes.
     */
    protected void removeAllActiveStates() {
        // categories
        moviesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        seriesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        favoritesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
    }

    /**
     * Get the UI settings from the application.
     *
     * @return Returns the ui settings.
     */
    protected UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }

    /**
     * Initialize the start screen of the application.
     *
     * @param startScreen The start screen that needs to be started.
     */
    private void initializeStartScreen(StartScreen startScreen) {
        log.trace("Initializing start screen");
        switch (startScreen) {
            case SERIES:
                switchCategory(seriesCategory);
                break;
            case FAVORITES:
                switchCategory(favoritesCategory);
                break;
            default:
                switchCategory(moviesCategory);
                break;
        }
    }

    //endregion
}

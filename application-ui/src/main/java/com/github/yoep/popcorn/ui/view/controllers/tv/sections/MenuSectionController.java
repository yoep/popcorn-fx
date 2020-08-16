package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.StartScreen;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.AbstractFilterSectionController;
import com.github.yoep.popcorn.ui.view.controls.DelayedTextField;
import com.github.yoep.popcorn.ui.view.models.Category;
import com.github.yoep.popcorn.ui.view.models.Genre;
import com.github.yoep.popcorn.ui.view.models.SortBy;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.core.io.ClassPathResource;

import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class MenuSectionController extends AbstractFilterSectionController implements Initializable {
    private static final String ACTIVE_STYLE_CLASS = "active";

    private final ApplicationEventPublisher eventPublisher;
    private final PopcornProperties properties;

    @FXML
    private Pane menuPane;
    @FXML
    private ImageView header;
    @FXML
    private Pane searchCategory;
    @FXML
    private Pane moviesCategory;
    @FXML
    private Pane seriesCategory;
    @FXML
    private Pane favoritesCategory;
    @FXML
    private Pane settingsItem;
    @FXML
    private Pane shutdownItem;
    @FXML
    private DelayedTextField searchField;

    public MenuSectionController(SettingsService settingsService, ApplicationEventPublisher eventPublisher, PopcornProperties properties) {
        super(settingsService);
        this.eventPublisher = eventPublisher;
        this.properties = properties;
    }

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeHeader();
        initializeSceneListener(menuPane);
        initializeSearch();
    }

    //endregion

    //region Functions

    private void initializeHeader() {
        var headerResource = new ClassPathResource("images/header-small.png");

        try {
            header.setImage(new Image(headerResource.getInputStream()));
        } catch (IOException ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void initializeSearch() {
        searchField.valueProperty().addListener((observable, oldValue, newValue) -> eventPublisher.publishEvent(new SearchEvent(this, newValue)));
    }

    @Override
    protected void initializeStartScreen(StartScreen startScreen) {
        log.trace("Initializing start screen");
        Pane category;

        switch (startScreen) {
            case SERIES:
                log.trace("Switching to series category");
                category = seriesCategory;
                break;
            case FAVORITES:
                log.trace("Switching to favorites category");
                category = favoritesCategory;
                break;
            default:
                log.trace("Switching to movies category");
                category = moviesCategory;
                break;
        }

        category.requestFocus();
        switchCategory(category);
    }

    private void switchCategory(Pane categoryPane) {
        var category = Category.MOVIES;

        moviesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        seriesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        favoritesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));

        categoryPane.getStyleClass().add(ACTIVE_STYLE_CLASS);

        if (categoryPane == seriesCategory) {
            category = Category.SERIES;
        }
        if (categoryPane == favoritesCategory) {
            category = Category.FAVORITES;
        }

        eventPublisher.publishEvent(new CategoryChangedEvent(this, category));
        updateGenres(category);
        updateSortBy(category);
        clearSearch();
    }

    //TODO: find a clever way to incorporate this into the UI
    private void updateGenres(Category category) {
        var providerProperties = properties.getProvider(category.getProviderName());
        var genre = new Genre(providerProperties.getGenres().get(0), null);

        eventPublisher.publishEvent(new GenreChangeEvent(this, genre));
    }

    //TODO: find a clever way to incorporate this into the UI
    private void updateSortBy(Category category) {
        var providerProperties = properties.getProvider(category.getProviderName());
        var sortBy = new SortBy(providerProperties.getSortBy().get(0), null);

        eventPublisher.publishEvent(new SortByChangeEvent(this, sortBy));
    }

    private void clearSearch() {
        searchField.setValue(null);
    }

    private void showSettings() {
        eventPublisher.publishEvent(new ShowSettingsEvent(this));
    }

    private void focusSearchField() {
        searchField.requestFocus();
    }

    private void onShutdown() {
        var stage = (Stage) shutdownItem.getScene().getWindow();

        stage.close();
    }

    @FXML
    private void onMoviesCategoryClicked(MouseEvent event) {
        event.consume();
        switchCategory(moviesCategory);
    }

    @FXML
    private void onSeriesCategoryClicked(MouseEvent event) {
        event.consume();
        switchCategory(seriesCategory);
    }

    @FXML
    private void onFavoritesCategoryClicked(MouseEvent event) {
        event.consume();
        switchCategory(favoritesCategory);
    }

    @FXML
    private void onSettingsClicked(MouseEvent event) {
        event.consume();
        showSettings();
    }

    @FXML
    private void onShutdownClicked(MouseEvent event) {
        event.consume();
        onShutdown();
    }

    @FXML
    private void onKeyEvent(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            var source = event.getSource();

            if (source == searchCategory) {
                focusSearchField();
            }
            if (source == moviesCategory || source == seriesCategory || source == favoritesCategory) {
                switchCategory((Pane) source);
            }
            if (source == settingsItem) {
                showSettings();
            }
            if (source == shutdownItem) {
                onShutdown();
            }
        }
    }

    //endregion
}

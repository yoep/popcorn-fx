package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SearchEvent;
import com.github.yoep.popcorn.ui.events.ShowSettingsEvent;
import com.github.yoep.popcorn.ui.events.SortByChangeEvent;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.AbstractFilterSectionController;
import com.github.yoep.popcorn.ui.view.controls.DelayedTextField;
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
    private final PopcornProperties properties;

    @FXML
    private Pane menuPane;
    @FXML
    private ImageView header;
    @FXML
    private Pane searchCategory;
    @FXML
    private Pane settingsItem;
    @FXML
    private Pane shutdownItem;
    @FXML
    private DelayedTextField searchField;

    public MenuSectionController(SettingsService settingsService, ApplicationEventPublisher eventPublisher, PopcornProperties properties) {
        super(eventPublisher, settingsService);
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

    @Override
    protected void clearSearch() {
        searchField.clear();
    }

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

    //TODO: find a clever way to incorporate this into the UI
    @Override
    protected void updateGenres(Category category) {
        var providerProperties = properties.getProvider(category.getProviderName());
        var genre = new Genre(providerProperties.getGenres().get(0), null);

        eventPublisher.publishEvent(new GenreChangeEvent(this, genre));
    }

    //TODO: find a clever way to incorporate this into the UI
    @Override
    protected void updateSortBy(Category category) {
        var providerProperties = properties.getProvider(category.getProviderName());
        var sortBy = new SortBy(providerProperties.getSortBy().get(0), null);

        eventPublisher.publishEvent(new SortByChangeEvent(this, sortBy));
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

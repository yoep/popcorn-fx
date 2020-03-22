package com.github.yoep.popcorn.view.controllers.tv.sections;

import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.StartScreen;
import com.github.yoep.popcorn.view.controllers.common.sections.AbstractFilterSectionController;
import com.github.yoep.popcorn.view.models.Category;
import com.github.yoep.popcorn.view.models.Genre;
import com.github.yoep.popcorn.view.models.SortBy;
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
import org.springframework.core.io.ClassPathResource;

import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class MenuSectionController extends AbstractFilterSectionController implements Initializable {
    private static final String ACTIVE_STYLE_CLASS = "active";

    private final ActivityManager activityManager;
    private final PopcornProperties properties;

    @FXML
    private Pane menuPane;
    @FXML
    private ImageView header;
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

    public MenuSectionController(ActivityManager activityManager, SettingsService settingsService, PopcornProperties properties) {
        super(settingsService);
        this.activityManager = activityManager;
        this.properties = properties;
    }

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeHeader();
        initializeSceneListener(menuPane);
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

    @Override
    protected void initializeStartScreen(StartScreen startScreen) {
        log.trace("Initializing start screen");

        switch (startScreen) {
            case SERIES:
                log.trace("Switching to series category");
                switchCategory(seriesCategory);
                break;
            case FAVORITES:
                log.trace("Switching to favorites category");
                switchCategory(favoritesCategory);
                break;
            default:
                log.trace("Switching to movies category");
                switchCategory(moviesCategory);
                break;
        }
    }

    private void switchCategory(Pane categoryPane) {
        final var category = new AtomicReference<>(Category.MOVIES);

        moviesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        seriesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        favoritesCategory.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));

        categoryPane.getStyleClass().add(ACTIVE_STYLE_CLASS);

        if (categoryPane == seriesCategory) {
            category.set(Category.SERIES);
        }
        if (categoryPane == favoritesCategory) {
            category.set(Category.FAVORITES);
        }

        activityManager.register((CategoryChangedActivity) category::get);
        updateGenres(category.get());
        updateSortBy(category.get());
    }

    //TODO: find a clever way to incorporate this into the UI
    private void updateGenres(Category category) {
        var providerProperties = properties.getProvider(category.getProviderName());

        activityManager.register((GenreChangeActivity) () -> new Genre(providerProperties.getGenres().get(0), null));
    }

    //TODO: find a clever way to incorporate this into the UI
    private void updateSortBy(Category category) {
        var providerProperties = properties.getProvider(category.getProviderName());

        activityManager.register((SortByChangeActivity) () -> new SortBy(providerProperties.getSortBy().get(0), null));
    }

    private void showSettings() {
        activityManager.register(new ShowSettingsActivity() {
        });
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

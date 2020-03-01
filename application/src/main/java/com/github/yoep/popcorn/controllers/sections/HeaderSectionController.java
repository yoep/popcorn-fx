package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.config.properties.ProviderProperties;
import com.github.yoep.popcorn.controls.SearchField;
import com.github.yoep.popcorn.controls.SearchListener;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.settings.models.TraktSettings;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;
import java.util.stream.Collectors;

@Slf4j
@Component
@RequiredArgsConstructor
public class HeaderSectionController implements Initializable {
    private static final String STYLE_ACTIVE = "active";

    private final ActivityManager activityManager;
    private final PopcornProperties properties;
    private final LocaleText localeText;
    private final SettingsService settingsService;

    @FXML
    private Label moviesCategory;
    @FXML
    private Label seriesCategory;
    @FXML
    private Label favoritesCategory;
    @FXML
    private ComboBox<Genre> genreCombo;
    @FXML
    private ComboBox<SortBy> sortByCombo;
    @FXML
    private SearchField search;
    @FXML
    private Icon watchlistIcon;
    @FXML
    private Icon torrentCollectionIcon;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeComboListeners();
        initializeCategory();
        initializeSearchListener();
        initializeIcons();
    }

    private void initializeComboListeners() {
        genreCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchGenre(newValue));
        sortByCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSortBy(newValue));
    }

    private void initializeCategory() {
        // set the default view to movies
        switchCategory(moviesCategory);
    }

    private void initializeSearchListener() {
        search.addListener(new SearchListener() {
            @Override
            public void onSearchValueChanged(String newValue) {
                activityManager.register((SearchActivity) () -> newValue);
            }

            @Override
            public void onSearchValueCleared() {
                activityManager.register((SearchActivity) () -> null);
            }
        });
    }

    private void initializeIcons() {
        var traktSettings = getSettings().getTraktSettings();

        watchlistIcon.setVisible(traktSettings.getAccessToken().isPresent());
        traktSettings.addListener(evt -> {
            if (evt.getPropertyName().equals(TraktSettings.ACCESS_TOKEN_PROPERTY)) {
                watchlistIcon.setVisible(traktSettings.getAccessToken().isPresent());
            }
        });
    }

    private void setGenres(Category category) {
        ProviderProperties providerProperties = properties.getProvider(category.getProviderName());
        List<Genre> genres = providerProperties.getGenres().stream()
                .map(e -> new Genre(e, localeText.get("genre_" + e)))
                .sorted()
                .collect(Collectors.toList());

        genreCombo.getItems().clear();
        genreCombo.getItems().addAll(genres);
        genreCombo.getSelectionModel().select(0);
    }

    private void setSortBy(Category category) {
        ProviderProperties providerProperties = properties.getProvider(category.getProviderName());
        List<SortBy> sortBy = providerProperties.getSortBy().stream()
                .map(e -> new SortBy(e, localeText.get("sort-by_" + e)))
                .collect(Collectors.toList());

        sortByCombo.getItems().clear();
        sortByCombo.getItems().addAll(sortBy);
        sortByCombo.getSelectionModel().select(0);
    }

    private void switchCategory(Label item) {
        final AtomicReference<Category> category = new AtomicReference<>();

        removeAllActiveStates();
        item.getStyleClass().add(STYLE_ACTIVE);

        if (item == moviesCategory) {
            category.set(Category.MOVIES);
        }
        if (item == seriesCategory) {
            category.set(Category.SERIES);
        }
        if (item == favoritesCategory) {
            category.set(Category.FAVORITES);
        }

        // invoke the chang activity first before changing the genre & sort by
        log.trace("Category is being changed to \"{}\"", category.get());
        activityManager.register((CategoryChangedActivity) category::get);

        // clear the current search
        search.clear();

        // set the category specific genres and sort by filters
        setGenres(category.get());
        setSortBy(category.get());
    }

    private void switchGenre(Genre genre) {
        log.trace("Genre is being changed to \"{}\"", genre);
        activityManager.register((GenreChangeActivity) () -> genre);
    }

    private void switchSortBy(SortBy sortBy) {
        log.trace("SortBy is being changed to \"{}\"", sortBy);
        activityManager.register((SortByChangeActivity) () -> sortBy);
    }

    private void switchIcon(Icon icon) {
        removeAllActiveStates();

        icon.getStyleClass().add(STYLE_ACTIVE);
    }

    private void removeAllActiveStates() {
        // categories
        moviesCategory.getStyleClass().removeIf(e -> e.equals(STYLE_ACTIVE));
        seriesCategory.getStyleClass().removeIf(e -> e.equals(STYLE_ACTIVE));
        favoritesCategory.getStyleClass().removeIf(e -> e.equals(STYLE_ACTIVE));

        // icons
        watchlistIcon.getStyleClass().removeIf(e -> e.equals(STYLE_ACTIVE));
        torrentCollectionIcon.getStyleClass().removeIf(e -> e.equals(STYLE_ACTIVE));
    }

    private ApplicationSettings getSettings() {
        return settingsService.getSettings();
    }

    @FXML
    private void onCategoryClicked(MouseEvent event) {
        switchCategory((Label) event.getSource());
    }

    @FXML
    private void onGenreClicked() {
        genreCombo.show();
    }

    @FXML
    private void onWatchlistClicked() {
        switchIcon(watchlistIcon);
        activityManager.register(new ShowWatchlistActivity() {
        });
    }

    @FXML
    private void onTorrentCollectionClicked() {
        switchIcon(torrentCollectionIcon);
        activityManager.register(new ShowTorrentCollectionActivity() {
        });
    }

    @FXML
    private void onSettingsClicked() {
        activityManager.register(new ShowSettingsActivity() {
        });
    }
}

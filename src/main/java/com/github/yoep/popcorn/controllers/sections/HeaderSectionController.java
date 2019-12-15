package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CategoryChangedActivity;
import com.github.yoep.popcorn.activities.GenreChangeActivity;
import com.github.yoep.popcorn.activities.SortByChangeActivity;
import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;
import java.util.stream.Collectors;

@Slf4j
@Controller
@RequiredArgsConstructor
public class HeaderSectionController implements Initializable {
    private static final String STYLE_ACTIVE = "active";

    private final ActivityManager activityManager;
    private final PopcornProperties popcornProperties;
    private final LocaleText localeText;

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

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeCategories();
        initializeGenres();
        initializeSortBy();
    }

    private void initializeCategories() {
        switchCategory(moviesCategory);
    }

    private void initializeGenres() {
        List<Genre> genres = popcornProperties.getGenres().stream()
                .map(e -> new Genre(e, localeText.get("genre_" + e)))
                .sorted()
                .collect(Collectors.toList());

        genreCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchGenre(newValue));
        genreCombo.getItems().addAll(genres);
        genreCombo.getSelectionModel().select(0);
    }

    private void initializeSortBy() {
        List<SortBy> sortBy = popcornProperties.getSortBy().stream()
                .map(e -> new SortBy(e, localeText.get("sort-by_" + e)))
                .collect(Collectors.toList());

        sortByCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) ->
                switchSortBy(newValue));
        sortByCombo.getItems().addAll(sortBy);
        sortByCombo.getSelectionModel().select(0);
    }

    private void switchCategory(Label item) {
        final AtomicReference<Category> category = new AtomicReference<>();

        moviesCategory.getStyleClass().remove(STYLE_ACTIVE);
        seriesCategory.getStyleClass().remove(STYLE_ACTIVE);
        favoritesCategory.getStyleClass().remove(STYLE_ACTIVE);

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

        log.trace("Category is being changed to \"{}\"", category.get());
        activityManager.register((CategoryChangedActivity) category::get);
    }

    private void switchGenre(Genre genre) {
        log.trace("Genre is being changed to \"{}\"", genre);
        activityManager.register((GenreChangeActivity) () -> genre);
    }

    private void switchSortBy(SortBy sortBy) {
        log.trace("SortBy is being changed to \"{}\"", sortBy);
        activityManager.register((SortByChangeActivity) () -> sortBy);
    }

    @FXML
    private void onCategoryClicked(MouseEvent event) {
        switchCategory((Label) event.getSource());
    }

    @FXML
    private void onGenreClicked() {
        genreCombo.show();
    }
}

package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.GenreChangeActivity;
import com.github.yoep.popcorn.activities.SortByChangeActivity;
import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.stream.Collectors;

@Controller
@RequiredArgsConstructor
public class HeaderSectionController implements Initializable {
    private final ActivityManager activityManager;
    private final PopcornProperties popcornProperties;
    private final LocaleText localeText;

    @FXML
    private ComboBox<Genre> genreCombo;
    @FXML
    private ComboBox<SortBy> sortByCombo;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeGenres();
        initializeSortBy();
    }

    private void initializeGenres() {
        List<Genre> genres = popcornProperties.getGenres().stream()
                .map(e -> new Genre(e, localeText.get("genre_" + e)))
                .sorted()
                .collect(Collectors.toList());

        genreCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) ->
                activityManager.register((GenreChangeActivity) () -> newValue));
        genreCombo.getItems().addAll(genres);
        genreCombo.getSelectionModel().select(0);
    }

    private void initializeSortBy() {
        List<SortBy> sortBy = popcornProperties.getSortBy().stream()
                .map(e -> new SortBy(e, localeText.get("sort-by_" + e)))
                .collect(Collectors.toList());

        sortByCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) ->
                activityManager.register((SortByChangeActivity) () -> newValue));
        sortByCombo.getItems().addAll(sortBy);
        sortByCombo.getSelectionModel().select(0);
    }

    @FXML
    private void onGenreClicked() {
        genreCombo.show();
    }
}

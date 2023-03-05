package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SortByChangeEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
@ViewController
public class FilterComponent implements Initializable {
    private final LocaleText localeText;
    private final EventPublisher eventPublisher;
    private final FxLib fxLib;
    private final PopcornFx instance;

    @FXML
    ComboBox<Genre> genreCombo;
    @FXML
    ComboBox<SortBy> sortByCombo;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        genreCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchGenre(newValue));
        sortByCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSortBy(newValue));
    }

    @PostConstruct
    void init() {
        eventPublisher.register(CategoryChangedEvent.class, this::onCategoryChanged);
    }

    private CategoryChangedEvent onCategoryChanged(CategoryChangedEvent event) {
        updateGenres(event.getCategory());
        updateSortBy(event.getCategory());
        return event;
    }

    private void updateGenres(Category category) {
        try (var libGenres = fxLib.retrieve_provider_genres(instance, category.getProviderName())) {
            var genres = libGenres.values().stream()
                    .map(e -> new Genre(e, localeText.get("genre_" + e)))
                    .sorted()
                    .toList();

            Platform.runLater(() -> {
                genreCombo.getItems().clear();
                genreCombo.getItems().addAll(genres);
                genreCombo.getSelectionModel().select(0);
            });
        }
    }

    private void updateSortBy(Category category) {
        try (var libSortBy = fxLib.retrieve_provider_sort_by(instance, category.getProviderName())) {
            var sortBy = libSortBy.values().stream()
                    .map(e -> new SortBy(e, localeText.get("sort-by_" + e)))
                    .toList();

            Platform.runLater(() -> {
                sortByCombo.getItems().clear();
                sortByCombo.getItems().addAll(sortBy);
                sortByCombo.getSelectionModel().select(0);
            });
        }
    }

    private void switchGenre(Genre genre) {
        if (genre == null)
            return;

        log.trace("Genre is being changed to \"{}\"", genre);
        eventPublisher.publish(new GenreChangeEvent(this, genre));
    }

    private void switchSortBy(SortBy sortBy) {
        if (sortBy == null)
            return;

        log.trace("SortBy is being changed to \"{}\"", sortBy);
        eventPublisher.publish(new SortByChangeEvent(this, sortBy));
    }

    @FXML
    void onGenreClicked(MouseEvent event) {
        event.consume();
        genreCombo.show();
    }

    @FXML
    void onSortClicked(MouseEvent event) {
        event.consume();
        sortByCombo.show();
    }
}

package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.VirtualKeyboard;
import javafx.animation.FadeTransition;
import javafx.animation.PauseTransition;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class TvFilterComponent implements Initializable {
    private final EventPublisher eventPublisher;
    private final LocaleText localeText;
    private final FxLib fxLib;
    private final PopcornFx instance;

    final FadeTransition slideAnimation = new FadeTransition(Duration.millis(500.0), new Pane());
    final PauseTransition searchTimeout = new PauseTransition(Duration.seconds(2));

    @FXML
    VBox filter;
    @FXML
    Label searchValue;
    @FXML
    VirtualKeyboard virtualKeyboard;
    @FXML
    AxisItemSelection<Genre> genres;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeSlideAnimation();
        initializeEvents();

        searchTimeout.setOnFinished(e -> eventPublisher.publish(new SearchEvent(this, virtualKeyboard.getText())));
        filter.focusWithinProperty().addListener((observable, oldValue, newValue) -> onFocusChanged(newValue));
        virtualKeyboard.textProperty().addListener((observable, oldValue, newValue) -> {
            searchValue.setText(newValue);
            searchTimeout.playFromStart();
        });
        genres.selectedItemProperty().addListener((observable, oldValue, newValue) -> onGenreChanged(newValue));

        onFocusChanged(false);
    }

    private void initializeSlideAnimation() {
        slideAnimation.setFromValue(0);
        slideAnimation.setToValue(1);
        slideAnimation.getNode().setOpacity(1.0);
        slideAnimation.getNode().opacityProperty().addListener((observable, oldValue, newValue) -> {
            filter.setMaxWidth(filter.getPrefWidth() * newValue.doubleValue());
        });
    }

    private void initializeEvents() {
        eventPublisher.register(RequestSearchFocus.class, event -> {
            Platform.runLater(() -> virtualKeyboard.requestFocus());
            return event;
        });
        eventPublisher.register(CategoryChangedEvent.class, this::onCategoryChanged);
    }

    private void onFocusChanged(boolean newValue) {
        if (newValue) {
            genres.setVvalue(0.0);
            slideAnimation.setFromValue(slideAnimation.getNode().getOpacity());
            slideAnimation.setToValue(1);
            slideAnimation.playFromStart();
        } else {
            slideAnimation.setFromValue(slideAnimation.getNode().getOpacity());
            slideAnimation.setToValue(0);
            slideAnimation.playFromStart();
        }
    }

    private CategoryChangedEvent onCategoryChanged(CategoryChangedEvent event) {
        virtualKeyboard.setText("");
        updateGenres(event.getCategory());
        updateSortBy(event.getCategory());
        return event;
    }

    private void onGenreChanged(Genre genre) {
        if (genre == null)
            return;

        eventPublisher.publish(new GenreChangeEvent(this, genre));
    }

    private void updateGenres(Category category) {
        try (var libGenres = fxLib.retrieve_provider_genres(instance, category.getProviderName())) {
            var values = libGenres.values().stream()
                    .map(e -> new Genre(e, localeText.get("genre_" + e)))
                    .sorted()
                    .toList();

            values.stream()
                    .findFirst()
                    .ifPresent(this::onGenreChanged);

            Platform.runLater(() -> genres.setItems(values.toArray(new Genre[0])));
        }
    }

    private void updateSortBy(Category category) {
        try (var libSortBy = fxLib.retrieve_provider_sort_by(instance, category.getProviderName())) {
            libSortBy.values().stream()
                    .findFirst()
                    .ifPresent(e -> eventPublisher.publish(new SortByChangeEvent(this, new SortBy(e, e))));
        }
    }
}

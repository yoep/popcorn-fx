package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.VirtualKeyboard;
import javafx.animation.FadeTransition;
import javafx.animation.PauseTransition;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import javafx.scene.layout.VBox;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class TvFilterComponent implements Initializable {
    private final FxChannel fxChannel;
    private final EventPublisher eventPublisher;
    private final LocaleText localeText;

    final FadeTransition slideAnimation = new FadeTransition(Duration.millis(500.0), new Pane());
    final PauseTransition searchTimeout = new PauseTransition(Duration.seconds(3));

    @FXML
    VBox filter;
    @FXML
    Label searchValue;
    @FXML
    VirtualKeyboard virtualKeyboard;
    @FXML
    AxisItemSelection<Media.Genre> genres;

    public TvFilterComponent(FxChannel fxChannel, EventPublisher eventPublisher, LocaleText localeText) {
        this.fxChannel = fxChannel;
        this.eventPublisher = eventPublisher;
        this.localeText = localeText;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeSlideAnimation();
        initializeEvents();

        searchTimeout.setOnFinished(e -> onSearch());
        filter.focusWithinProperty().addListener((observable, oldValue, newValue) -> onFocusChanged(newValue));
        virtualKeyboard.textProperty().addListener((observable, oldValue, newValue) -> {
            searchValue.setText(newValue);
            searchTimeout.playFromStart();
        });

        genres.selectedItemProperty().addListener((observable, oldValue, newValue) -> onGenreChanged(newValue));
        genres.setItemFactory(item -> new Button(item.getText()));

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

    private void onSearch() {
        var value = virtualKeyboard.getText();

        if (value.length() == 0 || value.length() >= 3) {
            eventPublisher.publish(new SearchEvent(this, virtualKeyboard.getText()));
        }
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

    private void onGenreChanged(Media.Genre genre) {
        if (genre == null)
            return;

        eventPublisher.publish(new GenreChangeEvent(this, genre));
    }

    private void updateGenres(Media.Category category) {
        fxChannel.send(GetCategoryGenresRequest.newBuilder()
                        .setCategory(category)
                        .build(), GetCategoryGenresResponse.parser())
                .whenComplete((response, throwable) -> {
                    if (throwable == null) {
                        var genres = response.getGenresList().stream()
                                .map(e -> Media.Genre.newBuilder(e)
                                        .setText(localeText.get("genre_" + e.getKey()))
                                        .build())
                                .toList();

                        Platform.runLater(() -> this.genres.setItems(genres.toArray(new Media.Genre[0])));
                    } else {
                        log.error("Failed to retrieve category genres, {}", throwable.getMessage(), throwable);
                    }
                });
    }

    private void updateSortBy(Media.Category category) {
        fxChannel.send(GetCategorySortByRequest.newBuilder()
                        .setCategory(category)
                        .build(), GetCategorySortByResponse.parser())
                .whenComplete((response, throwable) -> {
                    if (throwable == null) {
                        response.getSortByList().stream()
                                .map(e -> Media.SortBy.newBuilder(e)
                                        .setText(localeText.get("sort-by_" + e.getKey()))
                                        .build())
                                .findFirst()
                                .ifPresent(sortBy -> eventPublisher.publish(new SortByChangeEvent(this, sortBy)));
                    } else {
                        log.error("Failed to retrieve category sort by, {}", throwable.getMessage(), throwable);
                    }
                });
    }
}

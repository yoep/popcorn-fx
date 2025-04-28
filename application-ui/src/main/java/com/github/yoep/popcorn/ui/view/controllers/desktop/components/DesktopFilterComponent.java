package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SortByChangeEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ListCell;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class DesktopFilterComponent implements Initializable {
    private final FxChannel fxChannel;
    private final LocaleText localeText;
    private final EventPublisher eventPublisher;

    @FXML
    ComboBox<Media.Genre> genreCombo;
    @FXML
    ComboBox<Media.SortBy> sortByCombo;

    public DesktopFilterComponent(FxChannel fxChannel, LocaleText localeText, EventPublisher eventPublisher) {
        this.fxChannel = fxChannel;
        this.localeText = localeText;
        this.eventPublisher = eventPublisher;
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeGenre();
        initializeSortBy();
        eventPublisher.register(CategoryChangedEvent.class, this::onCategoryChanged);
    }

    private void initializeSortBy() {
        sortByCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSortBy(newValue));
        sortByCombo.setCellFactory(view -> new ListCell<>() {
            @Override
            protected void updateItem(Media.SortBy item, boolean empty) {
                super.updateItem(item, empty);
                setText(empty ? null : localeText.get("sort-by_" + item.getKey()));
            }
        });
        sortByCombo.setButtonCell(new ListCell<>() {
            @Override
            protected void updateItem(Media.SortBy item, boolean empty) {
                super.updateItem(item, empty);
                setText(empty ? null : localeText.get("sort-by_" + item.getKey()));
            }
        });
    }

    private void initializeGenre() {
        genreCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchGenre(newValue));
        genreCombo.setCellFactory(view -> new ListCell<>() {
            @Override
            protected void updateItem(Media.Genre item, boolean empty) {
                super.updateItem(item, empty);
                setText(empty ? null : localeText.get("genre_" + item.getKey()));
            }
        });
        genreCombo.setButtonCell(new ListCell<>() {
            @Override
            protected void updateItem(Media.Genre item, boolean empty) {
                super.updateItem(item, empty);
                setText(empty ? null : localeText.get("genre_" + item.getKey()));
            }
        });
    }

    private CategoryChangedEvent onCategoryChanged(CategoryChangedEvent event) {
        updateGenres(event.getCategory());
        updateSortBy(event.getCategory());
        return event;
    }

    private void updateGenres(Media.Category category) {
        fxChannel.send(GetCategoryGenresRequest.newBuilder()
                .setCategory(category)
                .build(), GetCategoryGenresResponse.parser()).thenAccept(response -> {
            if (response.getResult() == Response.Result.OK) {
                Platform.runLater(() -> {
                    genreCombo.getItems().clear();
                    genreCombo.getItems().addAll(response.getGenresList());
                    genreCombo.getSelectionModel().select(0);
                });
            } else {
                log.error("Failed to retrieve category genre options, {}", response.getError());
            }
        });
    }

    private void updateSortBy(Media.Category category) {
        fxChannel.send(GetCategorySortByRequest.newBuilder()
                        .setCategory(category)
                        .build(), GetCategorySortByResponse.parser())
                .thenAccept(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        Platform.runLater(() -> {
                            sortByCombo.getItems().clear();
                            sortByCombo.getItems().addAll(response.getSortByList());
                            sortByCombo.getSelectionModel().select(0);
                        });
                    } else {
                        log.error("Failed to retrieve category sort-by options, {}", response.getError());
                    }
                });
    }

    private void switchGenre(Media.Genre genre) {
        if (genre == null)
            return;

        log.trace("Genre is being changed to \"{}\"", genre);
        eventPublisher.publish(new GenreChangeEvent(this, genre));
    }

    private void switchSortBy(Media.SortBy sortBy) {
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

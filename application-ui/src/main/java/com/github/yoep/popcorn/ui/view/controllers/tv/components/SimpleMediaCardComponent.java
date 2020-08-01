package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.view.controllers.common.SimpleItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.AbstractMediaCardComponent;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.input.KeyCode;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
public class SimpleMediaCardComponent extends AbstractMediaCardComponent {
    private static final String WATCHED_STYLE_CLASS = "watched";

    private final List<SimpleItemListener> listeners = new ArrayList<>();
    private boolean requestFocus;

    @FXML
    private Pane posterItem;

    public SimpleMediaCardComponent(Media media, LocaleText localeText, ImageService imageService, SimpleItemListener... listeners) {
        super(media, localeText, imageService);
        this.listeners.addAll(asList(listeners));
    }

    public void setRequestFocus(boolean requestFocus) {
        this.requestFocus = requestFocus;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializeWatched();
        initializeKeyEvents();
        initializeRequestFocus();
    }

    private void initializeWatched() {
        switchWatched(media.isWatched());
        media.watchedProperty().addListener((observable, oldValue, newValue) -> switchWatched(newValue));
    }

    private void initializeRequestFocus() {
        if (requestFocus) {
            Platform.runLater(() -> posterItem.requestFocus());
        }
    }

    private void initializeKeyEvents() {
        posterItem.setOnKeyPressed(event -> {
            if (event.getCode() == KeyCode.ENTER) {
                event.consume();
                showDetails();
            }
        });
    }

    private void switchWatched(boolean isWatched) {
        if (isWatched) {
            posterItem.getStyleClass().add(WATCHED_STYLE_CLASS);
        } else {
            posterItem.getStyleClass().remove(WATCHED_STYLE_CLASS);
        }
    }

    @FXML
    private void showDetails() {
        listeners.forEach(listener -> listener.onClicked(media));
    }
}

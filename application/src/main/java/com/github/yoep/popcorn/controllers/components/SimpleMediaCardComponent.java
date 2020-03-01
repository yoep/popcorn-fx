package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.media.providers.models.Media;
import javafx.fxml.FXML;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

public class SimpleMediaCardComponent extends AbstractMediaCardComponent {
    private final List<SimpleItemListener> listeners = new ArrayList<>();

    public SimpleMediaCardComponent(Media media, LocaleText localeText, SimpleItemListener... listeners) {
        super(media, localeText);
        this.listeners.addAll(asList(listeners));
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
    }

    @FXML
    private void showDetails() {
        listeners.forEach(listener -> listener.onClicked(media));
    }
}
package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.RequestSearchFocus;
import com.github.yoep.popcorn.ui.events.SearchEvent;
import com.github.yoep.popcorn.ui.view.controls.SearchListener;
import com.github.yoep.popcorn.ui.view.controls.SearchTextField;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class DesktopSidebarSearchComponent implements Initializable {
    private final EventPublisher eventPublisher;

    @FXML
    SearchTextField searchInput;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeEvents();
        searchInput.addListener(new SearchListener() {
            @Override
            public void onSearchValueChanged(String newValue) {
                eventPublisher.publish(new SearchEvent(this, newValue));
            }

            @Override
            public void onSearchValueCleared() {
                eventPublisher.publish(new SearchEvent(this, null));
            }
        });
    }

    private void initializeEvents() {
        eventPublisher.register(CategoryChangedEvent.class, event -> {
            Platform.runLater(() -> searchInput.clear());
            return event;
        });
        eventPublisher.register(RequestSearchFocus.class, event -> {
            Platform.runLater(() -> searchInput.requestFocus());
            return event;
        });
    }
}

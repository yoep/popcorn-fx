package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStartedEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.HPos;
import javafx.scene.control.Label;
import javafx.scene.layout.GridPane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class PlayerHeaderComponent implements Initializable {
    static final String VIEW_HEADER_ACTIONS = "components/header-actions.component.fxml";

    private final EventPublisher eventPublisher;
    private final ViewLoader viewLoader;

    @FXML
    GridPane playerHeader;
    @FXML
    Label title;

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeMode();
        initializeListeners();
    }

    private void initializeMode() {
        var actions = viewLoader.load(VIEW_HEADER_ACTIONS);
        GridPane.setHalignment(actions, HPos.RIGHT);
        playerHeader.add(actions, 2, 0);
    }

    private void initializeListeners() {
        eventPublisher.register(PlayerStartedEvent.class, event -> {
            Platform.runLater(() -> title.setText(event.getTitle()));
            return event;
        });
    }

    //endregion
}

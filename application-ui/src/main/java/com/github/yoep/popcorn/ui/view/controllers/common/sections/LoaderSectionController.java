package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.LoadingStartedEvent;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class LoaderSectionController implements Initializable {
    private final ViewLoader viewLoader;
    private final EventPublisher eventPublisher;

    @FXML
    Pane rootPane;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePanes();
        eventPublisher.register(LoadingStartedEvent.class, event -> event);
    }

    private void initializePanes() {
        new Thread(() -> rootPane.getChildren().add(viewLoader.load("common/components/loader.component.fxml")),
                "LoaderSectionController.initialize").start();
    }
}

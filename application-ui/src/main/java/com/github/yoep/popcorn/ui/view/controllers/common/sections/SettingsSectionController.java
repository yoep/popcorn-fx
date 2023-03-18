package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.CloseSettingsEvent;
import javafx.fxml.FXML;
import javafx.scene.input.*;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class SettingsSectionController {
    private final EventPublisher eventPublisher;

    @FXML
    Pane settings;

    @FXML
    void onSettingsPressed(InputEvent event) {
        if (event instanceof KeyEvent keyEvent) {
            if (keyEvent.getCode() == KeyCode.ESCAPE || keyEvent.getCode() == KeyCode.BACK_SPACE) {
                event.consume();
                eventPublisher.publishEvent(new CloseSettingsEvent(this));
            }
        } else if (event instanceof MouseEvent mouseEvent) {
            if (mouseEvent.getButton() == MouseButton.BACK) {
                event.consume();
                eventPublisher.publishEvent(new CloseSettingsEvent(this));
            }
        }
    }
}

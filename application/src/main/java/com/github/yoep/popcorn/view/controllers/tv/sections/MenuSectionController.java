package com.github.yoep.popcorn.view.controllers.tv.sections;

import javafx.fxml.FXML;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;

public class MenuSectionController {
    @FXML
    private Pane shutdownItem;

    private void onShutdown() {
        var stage = (Stage) shutdownItem.getScene().getWindow();

        stage.close();
    }

    @FXML
    private void onMouseEvent(MouseEvent event) {
        if (event.getSource() == shutdownItem) {
            event.consume();

            onShutdown();
        }
    }

    @FXML
    private void onKeyEvent(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();

            if (event.getSource() == shutdownItem) {
                onShutdown();
            }
        }
    }
}

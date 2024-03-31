package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.controls.VirtualKeyboard;
import javafx.animation.PauseTransition;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class TvSettingsServerComponent implements Initializable {
    private final ApplicationConfig applicationConfig;

    final PauseTransition saveTimeout = new PauseTransition(Duration.seconds(5));

    @FXML
    Button apiServerBtn;
    @FXML
    Label apiServerTxt;
    @FXML
    VirtualKeyboard apiServerVirtualKeyboard;
    @FXML
    Overlay apiServerOverlay;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeText();
        apiServerVirtualKeyboard.textProperty().addListener((observable, oldValue, newValue) -> {
            apiServerBtn.setText(newValue);
            apiServerTxt.setText(newValue);
            saveTimeout.playFromStart();
        });
        saveTimeout.setOnFinished(this::onSave);
    }

    private void initializeText() {
        var apiServerValue = applicationConfig.getSettings().getServerSettings().getApiServer();
        apiServerBtn.setText(apiServerValue);
        apiServerTxt.setText(apiServerValue);
    }

    private void onSave(ActionEvent event) {
        var settings = applicationConfig.getSettings().getServerSettings();
        settings.setApiServer(apiServerVirtualKeyboard.getText());
        applicationConfig.update(settings);
    }

    @FXML
    void onCloseApiOverlay(ActionEvent event) {
        event.consume();
        apiServerOverlay.hide();
    }
}

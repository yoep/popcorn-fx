package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.tracking.TrackingService;
import com.github.yoep.popcorn.ui.trakt.TraktService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class SettingsTraktComponent implements Initializable {
    private final TraktService traktService;
    private final TrackingService trackingService;

    @FXML
    Label statusText;
    @FXML
    Button connectButton;
    @FXML
    Button disconnectButton;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeStatus();
        initializeButton();
    }

    private void initializeStatus() {
        switchText(traktService.isAuthorized());
    }

    private void initializeButton() {
        switchButtons(traktService.isAuthorized());
    }

    private void switchText(boolean isAuthorized) {

    }

    private void switchButtons(boolean isAuthorized) {
        connectButton.setVisible(!isAuthorized);
        disconnectButton.setVisible(isAuthorized);
    }

    @FXML
    private void onConnectClicked() {
        trackingService.authorize();
    }

    @FXML
    private void onDisconnectClicked() {
        traktService.forget();
        switchButtons(false);
    }
}

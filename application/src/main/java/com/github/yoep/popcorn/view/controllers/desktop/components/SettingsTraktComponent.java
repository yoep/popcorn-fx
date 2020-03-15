package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.yoep.popcorn.trakt.TraktService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@Component
@RequiredArgsConstructor
public class SettingsTraktComponent implements Initializable {
    private final TraktService traktService;

    @FXML
    private Label statusText;
    @FXML
    private Button connectButton;
    @FXML
    private Button disconnectButton;

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
        traktService.authorize().whenComplete((authorized, throwable) -> {
            if (throwable == null) {
                switchButtons(authorized);
            } else {
                log.error("Trakt.tv authorization failed, " + throwable.getMessage(), throwable);
            }
        });
    }

    @FXML
    private void onDisconnectClicked() {
        traktService.forget();
        switchButtons(false);
    }
}

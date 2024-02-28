package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.tracking.TrackingService;
import javafx.application.Platform;
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
    static final String AUTHORIZE_KEY = "settings_trakt_connect";
    static final String DISCONNECT_KEY = "settings_trakt_disconnect";
    static final String AUTHORIZE_ICON = Icon.LINK_UNICODE;
    static final String DISCONNECT_ICON = Icon.CHAIN_BROKEN_UNICODE;

    private final TrackingService trackingService;
    private final LocaleText localeText;

    @FXML
    Label statusText;
    @FXML
    Button authorizeBtn;
    @FXML
    Icon authorizeIcn;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        trackingService.addListener(isAuthorized -> Platform.runLater(() -> updateState(isAuthorized)));
        updateState(trackingService.isAuthorized());
    }

    private void updateState(boolean isAuthorized) {
        authorizeBtn.setText(localeText.get(isAuthorized ? DISCONNECT_KEY : AUTHORIZE_KEY));
        authorizeIcn.setText(isAuthorized ? DISCONNECT_ICON : AUTHORIZE_ICON);
    }

    @FXML
    private void onAuthorizeClicked() {
        if (trackingService.isAuthorized()) {
            trackingService.disconnect();
        } else {
            trackingService.authorize();
        }
    }
}

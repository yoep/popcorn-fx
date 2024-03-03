package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.tracking.TrackingService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.ApplicationConfigEvent;
import com.github.yoep.popcorn.backend.settings.models.TrackingSettings;
import com.github.yoep.popcorn.backend.settings.models.TrackingSyncState;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class SettingsTraktComponent implements Initializable {
    static final String AUTHORIZE_ICON = Icon.LINK_UNICODE;
    static final String DISCONNECT_ICON = Icon.CHAIN_BROKEN_UNICODE;

    private final ApplicationConfig applicationConfig;
    private final TrackingService trackingService;
    private final LocaleText localeText;

    @FXML
    Label statusText;
    @FXML
    Button authorizeBtn;
    @FXML
    Icon authorizeIcn;
    @FXML
    Label syncState;
    @FXML
    Label syncTime;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        trackingService.addListener(isAuthorized -> Platform.runLater(() -> updateState(isAuthorized)));
        applicationConfig.register(event -> {
            if (event.getTag() == ApplicationConfigEvent.Tag.TRACKING_SETTINGS_CHANGED) {
                Platform.runLater(() -> updateTrackingState(event.getUnion().getTrackingSettingsChanged_body().getSettings()));
            }
        });
        updateState(trackingService.isAuthorized());
    }

    private void updateState(boolean isAuthorized) {
        var settings = applicationConfig.getSettings();

        authorizeBtn.setText(localeText.get(isAuthorized ? SettingsMessage.DISCONNECT : SettingsMessage.AUTHORIZE));
        authorizeIcn.setText(isAuthorized ? DISCONNECT_ICON : AUTHORIZE_ICON);
        syncState.setVisible(isAuthorized);
        syncTime.setVisible(isAuthorized);
        updateTrackingState(settings.getTrackingSettings());
    }

    private void updateTrackingState(TrackingSettings trackingSettings) {
        syncState.setText(trackingSettings.getLastSync()
                .map(e -> localeText.get(SettingsMessage.LAST_SYNC_STATE,
                        localeText.get(e.getState() == TrackingSyncState.SUCCESS ? SettingsMessage.SYNC_SUCCESS : SettingsMessage.SYNC_FAILED)))
                .orElse(null));
        syncTime.setText(trackingSettings.getLastSync()
                .map(e -> localeText.get(SettingsMessage.LAST_SYNC_TIME, e.getTime()))
                .orElse(null));
    }

    private void onAuthorizationBtnAction() {
        if (trackingService.isAuthorized()) {
            trackingService.disconnect();
        } else {
            trackingService.authorize();
        }
    }

    @FXML
    void onAuthorizeClicked(MouseEvent event) {
        event.consume();
        onAuthorizationBtnAction();
    }

    @FXML
    void onAuthorizationPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onAuthorizationBtnAction();
        }
    }
}

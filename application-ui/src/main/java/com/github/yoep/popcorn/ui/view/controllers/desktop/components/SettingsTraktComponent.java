package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.media.tracking.TrackingService;
import com.github.yoep.popcorn.backend.settings.AbstractApplicationSettingsEventListener;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
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
import java.time.Instant;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class SettingsTraktComponent implements Initializable {
    static final String AUTHORIZE_ICON = Icon.LINK_UNICODE;
    static final String DISCONNECT_ICON = Icon.CHAIN_BROKEN_UNICODE;
    private static final DateTimeFormatter DATETIME_FORMATTER = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss");

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
        applicationConfig.addListener(new AbstractApplicationSettingsEventListener() {
            @Override
            public void onTrackingSettingsChanged(ApplicationSettings.TrackingSettings settings) {
                updateTrackingState(settings);
            }
        });
        trackingService.isAuthorized().thenAccept(this::updateState);
    }

    private void updateState(boolean isAuthorized) {
        applicationConfig.getSettings().thenAccept(settings -> {
            Platform.runLater(() -> {
                authorizeBtn.setText(localeText.get(isAuthorized ? SettingsMessage.DISCONNECT : SettingsMessage.AUTHORIZE));
                authorizeIcn.setText(isAuthorized ? DISCONNECT_ICON : AUTHORIZE_ICON);
                syncState.setVisible(isAuthorized);
                syncTime.setVisible(isAuthorized);
            });

            updateTrackingState(settings.getTrackingSettings());
        });
    }

    private void updateTrackingState(ApplicationSettings.TrackingSettings trackingSettings) {
        var lastSync = Optional.ofNullable(trackingSettings.getLastSync());
        Platform.runLater(() -> {
            syncState.setText(lastSync
                    .map(e -> localeText.get(SettingsMessage.LAST_SYNC_STATE,
                            localeText.get(e.getState() == ApplicationSettings.TrackingSettings.LastSync.State.SUCCESS
                                    ? SettingsMessage.SYNC_SUCCESS
                                    : SettingsMessage.SYNC_FAILED)))
                    .orElse(null));
            syncTime.setText(lastSync
                    .map(e -> Instant.ofEpochMilli(e.getLastSyncedMillis())
                            .atZone(ZoneId.systemDefault())
                            .toLocalDateTime())
                    .map(e -> localeText.get(SettingsMessage.LAST_SYNC_TIME, DATETIME_FORMATTER.format(e)))
                    .orElse(null));
        });
    }

    private void onAuthorizationBtnAction() {
        trackingService.isAuthorized().thenAccept(isAuthorized -> {
            if (isAuthorized) {
                trackingService.disconnect();
            } else {
                trackingService.authorize();
            }
        });
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

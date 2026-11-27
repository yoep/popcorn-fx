package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.controls.VirtualKeyboard;
import com.github.yoep.popcorn.ui.view.services.TorrentSettingsHelper;
import javafx.animation.PauseTransition;
import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Arrays;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
@RequiredArgsConstructor
public class TvSettingsTorrentComponent implements Initializable {
    static final String CLEANING_MODE_PREFIX = "cleaning_mode_";

    private final ApplicationConfig applicationConfig;
    private final LocaleText localeText;

    @FXML
    Button cacheCleanup;
    @FXML
    Overlay cacheCleanupOverlay;
    @FXML
    AxisItemSelection<ApplicationSettings.TorrentSettings.CleaningMode> cleanupModes;
    @FXML
    Button downloadLimitBtn;
    @FXML
    Overlay downloadLimitOverlay;
    @FXML
    Label downloadLimitTxt;
    @FXML
    VirtualKeyboard downloadLimitInput;
    @FXML
    Button uploadLimitBtn;
    @FXML
    Overlay uploadLimitOverlay;
    @FXML
    Label uploadLimitTxt;
    @FXML
    VirtualKeyboard uploadLimitInput;
    @FXML
    Button connectionLimitBtn;
    @FXML
    Overlay connectionLimitOverlay;
    @FXML
    Label connectionLimitTxt;
    @FXML
    VirtualKeyboard connectionLimitInput;

    final PauseTransition saveTimeout = new PauseTransition(Duration.seconds(2));

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeCleaningMode();
        initializeLimits();
        saveTimeout.setOnFinished(e -> onSaveLimits());
    }

    private void initializeLimits() {
        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            setLimit(downloadLimitBtn, downloadLimitTxt, downloadLimitInput, TorrentSettingsHelper.toDisplayValue(settings.getDownloadRateLimit()));
            setLimit(uploadLimitBtn, uploadLimitTxt, uploadLimitInput, TorrentSettingsHelper.toDisplayValue(settings.getUploadRateLimit()));
            setLimit(connectionLimitBtn, connectionLimitTxt, connectionLimitInput, String.valueOf(settings.getConnectionsLimit()));

            bindLimit(downloadLimitBtn, downloadLimitTxt, downloadLimitInput);
            bindLimit(uploadLimitBtn, uploadLimitTxt, uploadLimitInput);
            bindLimit(connectionLimitBtn, connectionLimitTxt, connectionLimitInput);
        }));
    }

    private void setLimit(Button button, Label label, VirtualKeyboard input, String value) {
        button.setText(value);
        label.setText(value);
        input.setText(value);
    }

    private void bindLimit(Button button, Label label, VirtualKeyboard input) {
        input.textProperty().addListener((observable, oldValue, newValue) -> {
            button.setText(newValue);
            label.setText(newValue);
            saveTimeout.playFromStart();
        });
    }

    private void onSaveLimits() {
        getSettings().thenAccept(settings -> {
            try {
                applicationConfig.update(ApplicationSettings.TorrentSettings.newBuilder(settings)
                        .setDownloadRateLimit(TorrentSettingsHelper.toSettingsValue(downloadLimitInput.getText().trim()))
                        .setUploadRateLimit(TorrentSettingsHelper.toSettingsValue(uploadLimitInput.getText().trim()))
                        .setConnectionsLimit(parseConnectionLimit(connectionLimitInput.getText()))
                        .build());
            } catch (NumberFormatException ex) {
                log.warn("Torrent limit is invalid, {}", ex.getMessage(), ex);
            }
        });
    }

    private static int parseConnectionLimit(String text) {
        var value = text.trim();
        return value.isEmpty() ? 0 : Integer.parseInt(value);
    }

    @FXML
    void onCloseInputOverlay(ActionEvent event) {
        event.consume();
        downloadLimitOverlay.hide();
        uploadLimitOverlay.hide();
        connectionLimitOverlay.hide();
        saveTimeout.stop();
        onSaveLimits();
    }

    private void initializeCleaningMode() {
        cleanupModes.setItemFactory(item -> new Button(localeText.get(CLEANING_MODE_PREFIX + item.name().toLowerCase())));
        cleanupModes.setItems(Arrays.stream(ApplicationSettings.TorrentSettings.CleaningMode.values())
                .filter(e -> e != ApplicationSettings.TorrentSettings.CleaningMode.UNRECOGNIZED)
                .toArray(ApplicationSettings.TorrentSettings.CleaningMode[]::new));
        cleanupModes.selectedItemProperty().addListener((observable, oldValue, newValue)
                -> onCleanupModeChanged(newValue));

        getSettings().thenAccept(settings ->
                cleanupModes.setSelectedItem(settings.getCleaningMode(), true));
    }

    private void onCleanupModeChanged(ApplicationSettings.TorrentSettings.CleaningMode newValue) {
        getSettings().thenAccept(settings -> {
            applicationConfig.update(ApplicationSettings.TorrentSettings.newBuilder(settings)
                    .setCleaningMode(newValue)
                    .build());
            cacheCleanup.setText(localeText.get(CLEANING_MODE_PREFIX + newValue.name().toLowerCase()));
            cacheCleanupOverlay.hide();
        });
    }

    private CompletableFuture<ApplicationSettings.TorrentSettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getTorrentSettings);
    }
}

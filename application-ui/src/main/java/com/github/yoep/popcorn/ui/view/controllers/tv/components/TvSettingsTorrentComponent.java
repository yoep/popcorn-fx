package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
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

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeCleaningMode();
    }

    private void initializeCleaningMode() {
        cleanupModes.setItemFactory(item -> new Button(localeText.get(CLEANING_MODE_PREFIX + item.name().toLowerCase())));
        cleanupModes.setItems(ApplicationSettings.TorrentSettings.CleaningMode.values());
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

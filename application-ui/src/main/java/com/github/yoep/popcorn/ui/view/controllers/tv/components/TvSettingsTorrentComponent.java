package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.CleaningMode;
import com.github.yoep.popcorn.backend.settings.models.TorrentSettings;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

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
    AxisItemSelection<CleaningMode> cleanupModes;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeCleaningMode();
    }

    private void initializeCleaningMode() {
        cleanupModes.setItemFactory(item -> new Button(localeText.get(CLEANING_MODE_PREFIX + item.name().toLowerCase())));
        cleanupModes.setItems(CleaningMode.values());
        cleanupModes.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setCleaningMode(newValue);
            applicationConfig.update(settings);
            cacheCleanup.setText(localeText.get(CLEANING_MODE_PREFIX + newValue.name().toLowerCase()));
            cacheCleanupOverlay.hide();
        });
        cleanupModes.setSelectedItem(getSettings().getCleaningMode(), true);
    }

    private TorrentSettings getSettings() {
        return applicationConfig.getSettings().getTorrentSettings();
    }
}

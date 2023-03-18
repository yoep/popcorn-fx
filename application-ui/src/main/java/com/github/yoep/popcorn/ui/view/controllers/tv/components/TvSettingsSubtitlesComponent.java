package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleFamily;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class TvSettingsSubtitlesComponent implements Initializable {
    private final ApplicationConfig applicationConfig;

    @FXML
    Button defaultSubtitle;
    @FXML
    AxisItemSelection<SubtitleLanguage> subtitles;
    @FXML
    Button fontFamily;
    @FXML
    AxisItemSelection<SubtitleFamily> fontFamilies;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        subtitles.setItems(SubtitleLanguage.values());
        subtitles.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setDefaultSubtitle(newValue);
            applicationConfig.update(settings);
            defaultSubtitle.setText(newValue.getNativeName());
        });
        subtitles.setSelectedItem(getSettings().getDefaultSubtitle(), true);

        fontFamilies.setItems(SubtitleFamily.values());
        fontFamilies.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setFontFamily(newValue);
            applicationConfig.update(settings);
            fontFamily.setText(newValue.getFamily());
        });
        fontFamilies.setSelectedItem(getSettings().getFontFamily());
    }

    public SubtitleSettings getSettings() {
        return applicationConfig.getSettings().getSubtitleSettings();
    }
}

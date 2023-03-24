package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleFamily;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
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
public class TvSettingsSubtitlesComponent implements Initializable {
    private final ApplicationConfig applicationConfig;

    @FXML
    Button defaultSubtitle;
    @FXML
    AxisItemSelection<SubtitleLanguage> subtitles;
    @FXML
    Overlay defaultSubtitleOverlay;
    @FXML
    Button fontFamily;
    @FXML
    AxisItemSelection<SubtitleFamily> fontFamilies;
    @FXML
    Overlay fontFamilyOverlay;
    @FXML
    Button fontSize;
    @FXML
    AxisItemSelection<Integer> fontSizes;
    @FXML
    Overlay fontSizeOverlay;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        subtitles.setItems(SubtitleLanguage.values());
        subtitles.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setDefaultSubtitle(newValue);
            applicationConfig.update(settings);
            defaultSubtitle.setText(newValue.getNativeName());
            defaultSubtitleOverlay.hide();
        });
        subtitles.setSelectedItem(getSettings().getDefaultSubtitle(), true);

        fontFamilies.setItems(SubtitleFamily.values());
        fontFamilies.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setFontFamily(newValue);
            applicationConfig.update(settings);
            fontFamily.setText(newValue.getFamily());
            fontFamilyOverlay.hide();
        });
        fontFamilies.setSelectedItem(getSettings().getFontFamily());

        fontSizes.setItems(SubtitleSettings.supportedFontSizes().toArray(Integer[]::new));
        fontSizes.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setFontSize(newValue);
            applicationConfig.update(settings);
            fontSize.setText(String.valueOf(newValue));
            fontSizeOverlay.hide();
        });
        fontSizes.setSelectedItem(getSettings().getFontSize());
    }

    public SubtitleSettings getSettings() {
        return applicationConfig.getSettings().getSubtitleSettings();
    }
}

package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
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
    private final LocaleText localeText;

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
    Button decoration;
    @FXML
    AxisItemSelection<DecorationType> decorations;
    @FXML
    Overlay decorationOverlay;
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
        initializeSubtitles();
        initializeFonts();
        initializeDecorations();

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

    private void initializeSubtitles() {
        subtitles.setItems(SubtitleLanguage.values());
        subtitles.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setDefaultSubtitle(newValue);
            applicationConfig.update(settings);
            defaultSubtitle.setText(newValue.getNativeName());
            defaultSubtitleOverlay.hide();
        });
        subtitles.setSelectedItem(getSettings().getDefaultSubtitle(), true);
    }

    private void initializeFonts() {
        fontFamilies.setItems(SubtitleFamily.values());
        fontFamilies.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setFontFamily(newValue);
            applicationConfig.update(settings);
            fontFamily.setText(newValue.getFamily());
            fontFamilyOverlay.hide();
        });
        fontFamilies.setSelectedItem(getSettings().getFontFamily());
    }

    private void initializeDecorations() {
        decorations.setItemFactory(item -> new Button(localeText.get("settings_subtitles_style_" + item.toString().toLowerCase())));
        decorations.setItems(DecorationType.values());
        decorations.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setDecoration(newValue);
            applicationConfig.update(settings);
            decoration.setText(localeText.get("settings_subtitles_style_" + newValue.toString().toLowerCase()));
            decorationOverlay.hide();
        });
        decorations.setSelectedItem(getSettings().getDecoration());
    }

    public SubtitleSettings getSettings() {
        return applicationConfig.getSettings().getSubtitleSettings();
    }
}

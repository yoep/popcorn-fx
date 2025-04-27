package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.application.Platform;
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
public class TvSettingsSubtitlesComponent implements Initializable {
    private final ApplicationConfig applicationConfig;
    private final LocaleText localeText;

    @FXML
    Button defaultSubtitle;
    @FXML
    AxisItemSelection<Subtitle.Language> subtitles;
    @FXML
    Overlay defaultSubtitleOverlay;
    @FXML
    Button fontFamily;
    @FXML
    AxisItemSelection<ApplicationSettings.SubtitleSettings.Family> fontFamilies;
    @FXML
    Button decoration;
    @FXML
    AxisItemSelection<ApplicationSettings.SubtitleSettings.DecorationType> decorations;
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

        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            fontSizes.setItems(SubtitleHelper.supportedFontSizes().toArray(Integer[]::new));
            fontSizes.setSelectedItem(settings.getFontSize());
            fontSizes.selectedItemProperty().addListener((observable, oldValue, newValue)
                    -> onFontSizeChanged(newValue));
        }));
    }

    private void initializeSubtitles() {
        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            subtitles.setItems(Subtitle.Language.values());
            subtitles.setSelectedItem(settings.getDefaultSubtitle(), true);
            subtitles.selectedItemProperty().addListener((observable, oldValue, newValue)
                    -> onDefaultSubtitleLanguageChanged(newValue));
        }));
    }

    private void initializeFonts() {
        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            fontFamilies.setItems(ApplicationSettings.SubtitleSettings.Family.values());
            fontFamilies.setSelectedItem(settings.getFontFamily());
            fontFamilies.selectedItemProperty().addListener((observable, oldValue, newValue)
                    -> onFontFamilyLanguageChanged(newValue));
        }));
    }

    private void initializeDecorations() {
        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            decorations.setItemFactory(item -> new Button(localeText.get("settings_subtitles_style_" + item.toString().toLowerCase())));
            decorations.setItems(ApplicationSettings.SubtitleSettings.DecorationType.values());
            decorations.setSelectedItem(settings.getDecoration());
            decorations.selectedItemProperty().addListener((observable, oldValue, newValue)
                    -> onDecorationTypeChanged(newValue));
        }));
    }

    private void onFontSizeChanged(Integer newValue) {
        getSettings().thenAccept(settings -> {
            applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                    .setFontSize(newValue)
                    .build());
            fontSize.setText(String.valueOf(newValue));
            fontSizeOverlay.hide();
        });
    }

    private void onDefaultSubtitleLanguageChanged(Subtitle.Language newValue) {
        getSettings().thenAccept(settings -> {
            applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                    .setDefaultSubtitle(newValue)
                    .build());
            defaultSubtitle.setText(SubtitleHelper.getNativeName(newValue));
            defaultSubtitleOverlay.hide();
        });
    }

    private void onFontFamilyLanguageChanged(ApplicationSettings.SubtitleSettings.Family newValue) {
        getSettings().thenAccept(settings -> {
            applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                    .setFontFamily(newValue)
                    .build());
            fontFamily.setText(newValue.name());
            fontFamilyOverlay.hide();
        });
    }

    private void onDecorationTypeChanged(ApplicationSettings.SubtitleSettings.DecorationType newValue) {
        getSettings().thenAccept(settings -> {
            applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder(settings)
                    .setDecoration(newValue)
                    .build());
            decoration.setText(localeText.get("settings_subtitles_style_" + newValue.toString().toLowerCase()));
            decorationOverlay.hide();
        });
    }

    public CompletableFuture<ApplicationSettings.SubtitleSettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getSubtitleSettings);
    }
}

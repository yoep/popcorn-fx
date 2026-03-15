package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Arrays;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class TvSettingsSubtitlesComponent extends AbstractSettingsComponent implements Initializable {
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

    public TvSettingsSubtitlesComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig applicationConfig) {
        super(eventPublisher, localeText, applicationConfig);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeSubtitles();
        initializeFonts();
        initializeDecorations();
        initializeSettings();
    }

    private void initializeSettings() {
        getSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> {
                    onSettingsLoaded(settings);
                    initializeListeners();
                });
            } else {
                log.error("Failed to retrieve settings", throwable);
                showErrorNotification(SettingsMessage.SETTINGS_FAILED_TO_LOAD);
            }
        });
    }

    private void initializeListeners() {
        subtitles.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            onDefaultSubtitleLanguageChanged(newValue);
            onSave();
        });
        fontFamilies.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            onFontFamilyLanguageChanged(newValue);
            onSave();
        });
        fontSizes.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            onFontSizeChanged(newValue);
            onSave();
        });
        decorations.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            onDecorationTypeChanged(newValue);
            onSave();
        });
    }

    private void initializeSubtitles() {
        subtitles.setItems(Arrays.stream(Subtitle.Language.values())
                .filter(e -> e != Subtitle.Language.UNRECOGNIZED)
                .toArray(Subtitle.Language[]::new));
    }

    private void initializeFonts() {
        fontFamilies.setItems(Arrays.stream(ApplicationSettings.SubtitleSettings.Family.values())
                .filter(e -> e != ApplicationSettings.SubtitleSettings.Family.UNRECOGNIZED)
                .toArray(ApplicationSettings.SubtitleSettings.Family[]::new));

        fontSizes.setItems(SubtitleHelper.supportedFontSizes().toArray(Integer[]::new));
    }

    private void initializeDecorations() {
        decorations.setItemFactory(item -> new Button(localeText.get("settings_subtitles_style_" + item.toString().toLowerCase())));
        decorations.setItems(Arrays.stream(ApplicationSettings.SubtitleSettings.DecorationType.values())
                .filter(e -> e != ApplicationSettings.SubtitleSettings.DecorationType.UNRECOGNIZED)
                .toArray(ApplicationSettings.SubtitleSettings.DecorationType[]::new));
    }

    private void onSettingsLoaded(ApplicationSettings.SubtitleSettings settings) {
        subtitles.setSelectedItem(settings.getDefaultSubtitle(), true);
        onDefaultSubtitleLanguageChanged(settings.getDefaultSubtitle());

        fontFamilies.setSelectedItem(settings.getFontFamily());
        onFontFamilyLanguageChanged(settings.getFontFamily());

        fontSizes.setSelectedItem(settings.getFontSize());
        onFontSizeChanged(settings.getFontSize());

        decorations.setSelectedItem(settings.getDecoration());
        onDecorationTypeChanged(settings.getDecoration());
    }

    private void onSave() {
        applicationConfig.update(ApplicationSettings.SubtitleSettings.newBuilder()
                .setDefaultSubtitle(subtitles.getSelectedItem())
                .setFontFamily(fontFamilies.getSelectedItem())
                .setFontSize(fontSizes.getSelectedItem())
                .setDecoration(decorations.getSelectedItem())
                .build());
        showNotification();
    }

    private void onFontSizeChanged(Integer newValue) {
        fontSize.setText(String.valueOf(newValue));
        fontSizeOverlay.hide();
    }

    private void onDefaultSubtitleLanguageChanged(Subtitle.Language newValue) {
        defaultSubtitle.setText(SubtitleHelper.getNativeName(newValue));
        defaultSubtitleOverlay.hide();
    }

    private void onFontFamilyLanguageChanged(ApplicationSettings.SubtitleSettings.Family newValue) {
        fontFamily.setText(newValue.name());
        fontFamilyOverlay.hide();
    }

    private void onDecorationTypeChanged(ApplicationSettings.SubtitleSettings.DecorationType newValue) {
        decoration.setText(localeText.get("settings_subtitles_style_" + newValue.toString().toLowerCase()));
        decorationOverlay.hide();
    }

    public CompletableFuture<ApplicationSettings.SubtitleSettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getSubtitleSettings);
    }
}

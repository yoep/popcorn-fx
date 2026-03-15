package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Locale;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class TvSettingsUiComponent extends AbstractSettingsUiComponent implements Initializable {
    @FXML
    Button defaultLanguage;
    @FXML
    Overlay defaultLanguageOverlay;
    @FXML
    AxisItemSelection<Locale> languages;
    @FXML
    Button uiScale;
    @FXML
    AxisItemSelection<UiScaleHolder> uiScales;
    @FXML
    Button startScreen;
    @FXML
    AxisItemSelection<Media.Category> startScreens;

    public TvSettingsUiComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig settingsService) {
        super(eventPublisher, localeText, settingsService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        requestFocus();
        initializeSettings();
    }

    private void requestFocus() {
        defaultLanguage.sceneProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                defaultLanguage.requestFocus();
            }
        });
    }

    private void initializeSettings() {
        languages.setItemFactory(item -> new Button(localeText.get("language_" + item.getLanguage())));
        languages.setItems(ApplicationConfig.supportedLanguages().toArray(Locale[]::new));

        uiScales.setItems(ApplicationConfig.supportedUIScales().stream()
                .map(UiScaleHolder::new)
                .toArray(UiScaleHolder[]::new));

        startScreens.setItemFactory(item -> new Button(localeText.get("filter_" + item.name().toLowerCase())));
        startScreens.setItems(startScreens());

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
        languages.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            saveSettings();
            onLanguageChanged(newValue);
            defaultLanguageOverlay.hide();
        });
        uiScales.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            saveSettings();
            onScaleChanged(newValue);
        });
        startScreens.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            onStartScreenChanged(newValue.name());
            saveSettings();
        });
    }

    private void onSettingsLoaded(ApplicationSettings.UISettings settings) {
        var language = Locale.forLanguageTag(settings.getDefaultLanguage());
        languages.setSelectedItem(language);
        onLanguageChanged(language);

        var scale = new UiScaleHolder(settings.getScale());
        uiScales.setSelectedItem(scale);
        onScaleChanged(scale);

        startScreens.setSelectedItem(settings.getStartScreen());
        onStartScreenChanged(settings.getStartScreen().name());
    }

    private void onLanguageChanged(Locale newValue) {
        defaultLanguage.setText(localeText.get("language_" + newValue.getLanguage()));
    }

    private void onScaleChanged(UiScaleHolder newValue) {
        var percentage = (int) (newValue.scale.getFactor() * 100);
        uiScale.setText(percentage + "%");
    }

    private void onStartScreenChanged(String newValue) {
        startScreen.setText(localeText.get("filter_" + newValue.toLowerCase()));
    }

    private void saveSettings() {
        applicationConfig.update(createSettings());
        showNotification();
    }

    private ApplicationSettings.UISettings createSettings() {
        return ApplicationSettings.UISettings.newBuilder()
                .setDefaultLanguage(languages.getSelectedItem().toString())
                .setStartScreen(startScreens.getSelectedItem())
                .setScale(uiScales.getSelectedItem().scale)
                .build();
    }

    private CompletableFuture<ApplicationSettings.UISettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getUiSettings);
    }

    private record UiScaleHolder(ApplicationSettings.UISettings.Scale scale) {
        @Override
        public String toString() {
            var percentage = (int) (scale.getFactor() * 100);
            return (percentage + "%");
        }
    }
}

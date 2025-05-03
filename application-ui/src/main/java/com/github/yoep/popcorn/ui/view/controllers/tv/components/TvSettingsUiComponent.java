package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
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
    AxisItemSelection<ApplicationSettings.UISettings.Scale> uiScales;
    @FXML
    Button startScreen;
    @FXML
    AxisItemSelection<Media.Category> startScreens;

    public TvSettingsUiComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig settingsService) {
        super(eventPublisher, localeText, settingsService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeListeners();
        initializeSettings();
    }

    private void initializeListeners() {
        defaultLanguage.sceneProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                defaultLanguage.requestFocus();
            }
        });
    }

    private void initializeSettings() {
        languages.setItemFactory(item -> new Button(localeText.get("language_" + item.toString())));
        languages.setItems(ApplicationConfig.supportedLanguages().toArray(Locale[]::new));
        languages.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            getSettings().whenComplete((settings, throwable) -> {
                if (throwable == null) {
                    applicationConfig.update(ApplicationSettings.UISettings.newBuilder(settings)
                            .setDefaultLanguage(newValue.toString())
                            .build());
                } else {
                    log.error("Failed to retrieve settings", throwable);
                }
            });
            defaultLanguage.setText(localeText.get("language_" + newValue));
            defaultLanguageOverlay.hide();
        });


        uiScales.setItems(ApplicationConfig.supportedUIScales().toArray(ApplicationSettings.UISettings.Scale[]::new));
        uiScales.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            getSettings().whenComplete((settings, throwable) -> {
                if (throwable == null) {
                    applicationConfig.update(ApplicationSettings.UISettings.newBuilder(settings)
                            .setScale(newValue)
                            .build());
                } else {
                    log.error("Failed to retrieve settings", throwable);
                }
            });
            uiScale.setText(newValue.toString());
        });

        initializeStartScreen();

        getSettings().thenAccept(settings -> Platform.runLater(() -> {
            languages.setSelectedItem(Locale.forLanguageTag(settings.getDefaultLanguage()));
            uiScales.setSelectedItem(settings.getScale());
            startScreens.setSelectedItem(settings.getStartScreen());
        }));
    }

    private void initializeStartScreen() {
        startScreens.setItemFactory(item -> new Button(localeText.get("filter_" + item.name().toLowerCase())));
        startScreens.setItems(startScreens());
        startScreens.selectedItemProperty().addListener((observable, oldValue, newValue) -> onStartScreenChanged(newValue));
    }

    private void onStartScreenChanged(Media.Category newValue) {
        getSettings().thenAccept(settings -> {
            applicationConfig.update(ApplicationSettings.UISettings.newBuilder(settings)
                    .setStartScreen(newValue)
                    .build());
        });
        startScreen.setText(localeText.get("filter_" + newValue.name().toLowerCase()));
    }

    private CompletableFuture<ApplicationSettings.UISettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getUiSettings);
    }
}

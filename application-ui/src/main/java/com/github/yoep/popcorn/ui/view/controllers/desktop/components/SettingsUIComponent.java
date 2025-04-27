package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Locale;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class SettingsUIComponent extends AbstractSettingsUiComponent implements Initializable {
    @FXML
    ComboBox<Locale> defaultLanguage;
    @FXML
    ComboBox<ApplicationSettings.UISettings.Scale> uiScale;
    @FXML
    ComboBox<Media.Category> startScreen;
    @FXML
    CheckBox nativeWindow;

    public SettingsUIComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig settingsService) {
        super(eventPublisher, localeText, settingsService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeDefaultLanguage();
        initializeUIScale();
        initializeStartScreen();
        initializeNativeWindow();
    }

    private void initializeDefaultLanguage() {
        defaultLanguage.setCellFactory(param -> createLanguageCell());
        defaultLanguage.setButtonCell(createLanguageCell());

        defaultLanguage.getItems().addAll(ApplicationConfig.supportedLanguages());
        getUiSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                defaultLanguage.getSelectionModel().select(Locale.forLanguageTag(settings.getDefaultLanguage()));
                defaultLanguage.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> updateLanguage(newValue));
                defaultLanguage.sceneProperty().addListener((observable, oldValue, newValue) -> {
                    if (newValue != null)
                        defaultLanguage.requestFocus();
                });
            } else {
                log.error("Failed to retrieve UI settings", throwable);
            }
        });
    }

    private void initializeUIScale() {
        uiScale.getItems().clear();
        uiScale.getItems().addAll(ApplicationConfig.supportedUIScales());
        getUiSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                uiScale.getSelectionModel().select(settings.getScale());
                uiScale.getSelectionModel().selectedItemProperty().addListener(((observable, oldValue, newValue) -> updateUIScale(newValue)));
            } else {
                log.error("Failed to retrieve UI settings", throwable);
            }
        });
    }

    private void initializeStartScreen() {
        startScreen.setCellFactory(param -> createStartScreenCell());
        startScreen.setButtonCell(createStartScreenCell());

        startScreen.getItems().addAll(Media.Category.values());
//        startScreen.getSelectionModel().select(getUiSettings().getStartScreen());
        startScreen.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> updateStartScreen(newValue));
    }

    private void initializeNativeWindow() {
//        nativeWindow.setSelected(getUiSettings().getNativeWindowEnabled());
        nativeWindow.selectedProperty().addListener((observableValue, oldValue, newValue) -> updateNativeWindow(newValue));
    }

    private void updateLanguage(Locale locale) {
        getUiSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                applicationConfig.update(ApplicationSettings.UISettings.newBuilder(settings)
                        .setDefaultLanguage(locale.toString())
                        .build());
                showNotification();
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
        //TODO: force the UI to reload to apply the text changes
    }

    private void updateUIScale(ApplicationSettings.UISettings.Scale newValue) {
        getUiSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                applicationConfig.update(ApplicationSettings.UISettings.newBuilder(settings)
                        .setScale(newValue)
                        .build());
                showNotification();
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    private void updateStartScreen(Media.Category category) {
        getUiSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                applicationConfig.update(ApplicationSettings.UISettings.newBuilder(settings)
                        .setStartScreen(category)
                        .build());
                showNotification();
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    private void updateNativeWindow(Boolean newValue) {
        getUiSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                applicationConfig.update(ApplicationSettings.UISettings.newBuilder(settings)
                        .setNativeWindowEnabled(newValue)
                        .build());
                showNotification();
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    private CompletableFuture<ApplicationSettings.UISettings> getUiSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getUiSettings);
    }
}

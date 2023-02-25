package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.StartScreen;
import com.github.yoep.popcorn.backend.settings.models.UIScale;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URL;
import java.util.Locale;
import java.util.ResourceBundle;

public class SettingsUIComponent extends AbstractSettingsUiComponent implements Initializable {
    @FXML
    ComboBox<Locale> defaultLanguage;
    @FXML
    ComboBox<UIScale> uiScale;
    @FXML
    ComboBox<StartScreen> startScreen;
    @FXML
    CheckBox nativeWindow;

    public SettingsUIComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, ApplicationConfig settingsService) {
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

        defaultLanguage.getItems().addAll(UISettings.supportedLanguages());
        defaultLanguage.getSelectionModel().select(Locale.forLanguageTag(getUiSettings().getDefaultLanguage()));
        defaultLanguage.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> updateLanguage(newValue));
    }

    private void initializeUIScale() {
        uiScale.getItems().clear();
        uiScale.getItems().addAll(ApplicationConfig.supportedUIScales());
        uiScale.getSelectionModel().select(getUiSettings().getUiScale());
        uiScale.getSelectionModel().selectedItemProperty().addListener(((observable, oldValue, newValue) -> updateUIScale(newValue)));
    }

    private void initializeStartScreen() {
        startScreen.setCellFactory(param -> createStartScreenCell());
        startScreen.setButtonCell(createStartScreenCell());

        startScreen.getItems().addAll(StartScreen.values());
        startScreen.getSelectionModel().select(getUiSettings().getStartScreen());
        startScreen.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> updateStartScreen(newValue));
    }

    private void initializeNativeWindow() {
        nativeWindow.setSelected(getUiSettings().isNativeWindowEnabled());
        nativeWindow.selectedProperty().addListener((observableValue, oldValue, newValue) -> updateNativeWindow(newValue));
    }

    private void updateLanguage(Locale locale) {
        getUiSettings().setDefaultLanguage(locale.toString());
        applicationConfig.update(getUiSettings());
        showNotification();
        //TODO: force the UI to reload to apply the text changes
    }

    private void updateUIScale(UIScale newValue) {
        getUiSettings().setUiScale(newValue);
        applicationConfig.update(getUiSettings());
        showNotification();
    }

    private void updateStartScreen(StartScreen startScreen) {
        getUiSettings().setStartScreen(startScreen);
        applicationConfig.update(getUiSettings());
        showNotification();
    }

    private void updateNativeWindow(Boolean newValue) {
        getUiSettings().setNativeWindowEnabled(newValue);
        applicationConfig.update(getUiSettings());
        showNotification();
    }

    private UISettings getUiSettings() {
        return applicationConfig.getSettings().getUiSettings();
    }
}

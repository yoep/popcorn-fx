package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.StartScreen;
import com.github.yoep.popcorn.ui.settings.models.UIScale;
import com.github.yoep.popcorn.ui.settings.models.UISettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ListCell;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URL;
import java.util.Locale;
import java.util.ResourceBundle;

public class SettingsUIComponent extends AbstractSettingsComponent implements Initializable {
    @FXML
    private ComboBox<Locale> defaultLanguage;
    @FXML
    private ComboBox<UIScale> uiScale;
    @FXML
    private ComboBox<StartScreen> startScreen;

    public SettingsUIComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, SettingsService settingsService) {
        super(eventPublisher, localeText, settingsService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeDefaultLanguage();
        initializeUIScale();
        initializeStartScreen();
    }

    private void initializeDefaultLanguage() {
        defaultLanguage.setCellFactory(param -> createLanguageCell());
        defaultLanguage.setButtonCell(createLanguageCell());

        defaultLanguage.getItems().addAll(UISettings.supportedLanguages());
        defaultLanguage.getSelectionModel().select(getUiSettings().getDefaultLanguage());
        defaultLanguage.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> updateLanguage(newValue));
    }

    private void initializeUIScale() {
        uiScale.getItems().clear();
        uiScale.getItems().addAll(SettingsService.supportedUIScales());
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

    private void updateLanguage(Locale locale) {
        getUiSettings().setDefaultLanguage(locale);
        showNotification();
        //TODO: force the UI to reload to apply the text changes
    }

    private void updateUIScale(UIScale newValue) {
        getUiSettings().setUiScale(newValue);
        showNotification();
    }

    private void updateStartScreen(StartScreen startScreen) {
        getUiSettings().setStartScreen(startScreen);
        showNotification();
    }

    private ListCell<Locale> createLanguageCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(Locale item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    setText(localeText.get("language_" + item.getLanguage()));
                } else {
                    setText(null);
                }
            }
        };
    }

    private ListCell<StartScreen> createStartScreenCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(StartScreen item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    setText(localeText.get("filter_" + item.name().toLowerCase()));
                } else {
                    setText(null);
                }
            }
        };
    }

    private UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }
}

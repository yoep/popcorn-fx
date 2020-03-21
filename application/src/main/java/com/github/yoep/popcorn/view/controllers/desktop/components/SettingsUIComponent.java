package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.SuccessNotificationActivity;
import com.github.yoep.popcorn.messages.SettingsMessage;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.StartScreen;
import com.github.yoep.popcorn.settings.models.UIScale;
import com.github.yoep.popcorn.settings.models.UISettings;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ListCell;
import lombok.RequiredArgsConstructor;

import java.net.URL;
import java.util.Locale;
import java.util.ResourceBundle;

@RequiredArgsConstructor
public class SettingsUIComponent implements Initializable {
    private final ActivityManager activityManager;
    private final SettingsService settingsService;
    private final LocaleText localeText;

    @FXML
    private ComboBox<Locale> defaultLanguage;
    @FXML
    private ComboBox<UIScale> uiScale;
    @FXML
    private ComboBox<StartScreen> startScreen;

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
        defaultLanguage.getSelectionModel().select(UISettings.defaultLanguage());
        defaultLanguage.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> updateLanguage(newValue));
    }

    private void initializeUIScale() {
        var items = uiScale.getItems();

        items.add(new UIScale(0.25f));
        items.add(new UIScale(0.5f));
        items.add(new UIScale(0.75f));
        items.add(new UIScale(1.0f));
        items.add(new UIScale(1.25f));
        items.add(new UIScale(1.50f));
        items.add(new UIScale(2.0f));
        items.add(new UIScale(3.0f));

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

    private void showNotification() {
        activityManager.register((SuccessNotificationActivity) () -> localeText.get(SettingsMessage.SETTINGS_SAVED));
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

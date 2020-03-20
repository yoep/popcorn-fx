package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.SuccessNotificationActivity;
import com.github.yoep.popcorn.messages.SettingsMessage;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.UIScale;
import com.github.yoep.popcorn.settings.models.UISettings;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;

import java.net.URL;
import java.util.Locale;
import java.util.ResourceBundle;
import java.util.stream.Collectors;

@Component
@RequiredArgsConstructor
public class SettingsUIComponent implements Initializable {
    private final ActivityManager activityManager;
    private final SettingsService settingsService;
    private final LocaleText localeText;

    @FXML
    private ComboBox<Language> defaultLanguage;
    @FXML
    private ComboBox<UIScale> uiScale;
    @FXML
    private ComboBox startScreen;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeDefaultLanguage();
        initializeUIScale();
    }

    private void initializeDefaultLanguage() {
        var languages = UISettings.supportedLanguages().stream()
                .map(this::toLanguage)
                .collect(Collectors.toList());
        var select = languages.stream()
                .filter(e -> e.getLocale().getLanguage().equals(getUiSettings().getDefaultLanguage().getLanguage()))
                .findFirst()
                .orElse(languages.get(0));

        defaultLanguage.getItems().addAll(languages);
        defaultLanguage.getSelectionModel().select(select);
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

    private Language toLanguage(Locale locale) {
        return new Language(locale, localeText.get("language_" + locale.getLanguage()));
    }

    private void updateLanguage(Language newValue) {
        getUiSettings().setDefaultLanguage(newValue.getLocale());
        showNotification();
        //TODO: force the UI to reload to apply the text changes
    }

    private void updateUIScale(UIScale newValue) {
        getUiSettings().setUiScale(newValue);
        showNotification();
    }

    private void showNotification() {
        activityManager.register((SuccessNotificationActivity) () -> localeText.get(SettingsMessage.SETTINGS_SAVED));
    }

    private UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }

    @Getter
    @EqualsAndHashCode
    private static class Language {
        private final Locale locale;
        private final String text;

        private Language(Locale locale, String text) {
            this.locale = locale;
            this.text = text;
        }

        @Override
        public String toString() {
            return text;
        }
    }
}

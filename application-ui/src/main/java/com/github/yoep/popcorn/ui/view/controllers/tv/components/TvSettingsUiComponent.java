package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.UIScale;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Locale;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class TvSettingsUiComponent implements Initializable {
    private final ApplicationConfig applicationConfig;
    private final LocaleText localeText;

    @FXML
    Button defaultLanguage;
    @FXML
    Overlay defaultLanguageOverlay;
    @FXML
    AxisItemSelection<Locale> languages;
    @FXML
    Button uiScale;
    @FXML
    AxisItemSelection<UIScale> uiScales;
    @FXML
    Button startScreen;
    @FXML
    AxisItemSelection<Category> startScreens;

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
        languages.setItems(UISettings.supportedLanguages().toArray(Locale[]::new));
        languages.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setDefaultLanguage(newValue.toString());
            applicationConfig.update(settings);
            defaultLanguage.setText(localeText.get("language_" + newValue));
            defaultLanguageOverlay.hide();
        });
        languages.setSelectedItem(Locale.forLanguageTag(getSettings().getDefaultLanguage()));

        uiScales.setItems(ApplicationConfig.supportedUIScales().toArray(UIScale[]::new));
        uiScales.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setUiScale(newValue);
            applicationConfig.update(settings);
            uiScale.setText(newValue.toString());
        });
        uiScales.setSelectedItem(getSettings().getUiScale());

        startScreens.setItemFactory(item -> new Button(localeText.get("filter_" + item.name().toLowerCase())));
        startScreens.setItems(Category.values());
        startScreens.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getSettings();
            settings.setStartScreen(newValue);
            applicationConfig.update(settings);
            startScreen.setText(localeText.get("filter_" + newValue.name().toLowerCase()));
        });
        startScreens.setSelectedItem(getSettings().getStartScreen());
    }

    private UISettings getSettings() {
        return applicationConfig.getSettings().getUiSettings();
    }
}

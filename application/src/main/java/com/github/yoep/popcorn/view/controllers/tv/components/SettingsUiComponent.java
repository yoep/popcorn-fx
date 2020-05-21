package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowSettingsActivity;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.UIScale;
import com.github.yoep.popcorn.settings.models.UISettings;
import com.github.yoep.popcorn.view.controllers.common.components.AbstractSettingsComponent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Locale;
import java.util.ResourceBundle;

@Slf4j
public class SettingsUiComponent extends AbstractSettingsComponent implements Initializable {

    @FXML
    private Pane defaultLanguageCombo;
    @FXML
    private Label defaultLanguage;
    @FXML
    private Label uiScale;
    @FXML
    private Label startScreen;

    //region Constructors

    public SettingsUiComponent(ActivityManager activityManager, LocaleText localeText, SettingsService settingsService) {
        super(activityManager, localeText, settingsService);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeDefaultLanguage();
        initializeUIScale();
    }

    private void initializeDefaultLanguage() {
        var uiSettings = getUiSettings();

        updateDefaultLanguage(uiSettings.getDefaultLanguage());
        uiSettings.addListener(evt -> {
            if (evt.getPropertyName().equalsIgnoreCase(UISettings.LANGUAGE_PROPERTY)) {
                updateDefaultLanguage((Locale) evt.getNewValue());
            }
        });
    }

    private void initializeUIScale() {
        var uiSettings = getUiSettings();

        updateUiScale(uiSettings.getUiScale());
        uiSettings.addListener(evt -> {
            if (evt.getPropertyName().equalsIgnoreCase(UISettings.UI_SCALE_PROPERTY)) {
                updateUiScale((UIScale) evt.getNewValue());
            }
        });
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowSettingsActivity.class, activity ->
                Platform.runLater(() -> defaultLanguageCombo.requestFocus()));
    }

    //endregion

    //region Functions

    private void updateDefaultLanguage(Locale language) {
        var text = localeText.get("language_" + language.getLanguage());

        defaultLanguage.setText(text);
    }

    private void updateUiScale(UIScale uiScale) {
        this.uiScale.setText(uiScale.toString());
    }

    private UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }

    //endregion
}

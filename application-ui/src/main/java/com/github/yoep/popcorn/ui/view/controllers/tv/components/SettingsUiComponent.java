package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ShowSettingsEvent;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.StartScreen;
import com.github.yoep.popcorn.ui.settings.models.UIScale;
import com.github.yoep.popcorn.ui.settings.models.UISettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;

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

    public SettingsUiComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, SettingsService settingsService) {
        super(eventPublisher, localeText, settingsService);
    }

    //endregion

    //region Methods

    @EventListener(ShowSettingsEvent.class)
    public void onShowSettings() {
        Platform.runLater(() -> defaultLanguageCombo.requestFocus());
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeDefaultLanguage();
        initializeUIScale();
        initializeStartScreen();
    }

    private void initializeDefaultLanguage() {
        var uiSettings = getUiSettings();

        updateDefaultLanguage(uiSettings.getDefaultLanguage());
        uiSettings.addListener(evt -> {
            if (evt.getPropertyName().equals(UISettings.LANGUAGE_PROPERTY)) {
                updateDefaultLanguage((Locale) evt.getNewValue());
            }
        });
    }

    private void initializeUIScale() {
        var uiSettings = getUiSettings();

        updateUiScale(uiSettings.getUiScale());
        uiSettings.addListener(evt -> {
            if (evt.getPropertyName().equals(UISettings.UI_SCALE_PROPERTY)) {
                updateUiScale((UIScale) evt.getNewValue());
            }
        });
    }

    private void initializeStartScreen() {
        var uiSettings = getUiSettings();

        updateStartScreen(uiSettings.getStartScreen());
        uiSettings.addListener(evt -> {
            if (evt.getPropertyName().equals(UISettings.START_SCREEN_PROPERTY)) {
                updateStartScreen((StartScreen) evt.getNewValue());
            }
        });
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

    private void updateStartScreen(StartScreen startScreen) {
        this.startScreen.setText(localeText.get("filter_" + startScreen.name().toLowerCase()));
    }

    private UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }

    //endregion
}

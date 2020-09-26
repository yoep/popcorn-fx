package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ShowSettingsEvent;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.StartScreen;
import com.github.yoep.popcorn.ui.settings.models.UIScale;
import com.github.yoep.popcorn.ui.settings.models.UISettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.sections.SettingsSectionController;
import javafx.application.Platform;
import javafx.beans.InvalidationListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.Locale;
import java.util.ResourceBundle;

@Slf4j
public class SettingsUiComponent extends AbstractSettingsComponent implements Initializable {
    private final SettingsSectionController settingsSection;

    @FXML
    private Pane defaultLanguageCombo;
    @FXML
    private Label defaultLanguage;
    @FXML
    private Label uiScale;
    @FXML
    private Label startScreen;
    private ListView<Locale> languages;

    //region Constructors

    public SettingsUiComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, SettingsService settingsService,
                               SettingsSectionController settingsSection) {
        super(eventPublisher, localeText, settingsService);
        this.settingsSection = settingsSection;
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
        initializeLanguages();
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

    private void initializeLanguages() {
        languages = new ListView<>();

        languages.setMaxWidth(150);
        languages.getItems().addListener((InvalidationListener) observable -> languages.setMaxHeight(50.0 * languages.getItems().size()));
        languages.getItems().addAll(UISettings.supportedLanguages());
        languages.setCellFactory(param -> new ListCell<>() {
            @Override
            protected void updateItem(Locale item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    setText(localeText.get("language_" + item.getLanguage()));
                } else {
                    setText(null);
                }
            }
        });
        languages.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var uiSettings = getUiSettings();
            uiSettings.setDefaultLanguage(newValue);
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

    private void onDefaultLanguageEvent() {
        settingsSection.showOverlay(defaultLanguageCombo, languages);
    }

    @FXML
    private void onDefaultLanguageKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onDefaultLanguageEvent();
        }
    }

    @FXML
    private void onDefaultLanguageClicked(MouseEvent event) {
        event.consume();
        onDefaultLanguageEvent();
    }

    //endregion
}

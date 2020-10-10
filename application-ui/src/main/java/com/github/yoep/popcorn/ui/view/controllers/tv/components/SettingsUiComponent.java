package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ShowSettingsEvent;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.StartScreen;
import com.github.yoep.popcorn.ui.settings.models.UIScale;
import com.github.yoep.popcorn.ui.settings.models.UISettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.sections.SettingsSectionController;
import javafx.application.Platform;
import javafx.beans.InvalidationListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
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
public class SettingsUiComponent extends AbstractSettingsUiComponent implements Initializable {
    private final SettingsSectionController settingsSection;

    @FXML
    private Pane defaultLanguageCombo;
    @FXML
    private Label defaultLanguage;
    @FXML
    private Pane uiScaleCombo;
    @FXML
    private Label uiScale;
    @FXML
    private Pane startScreenCombo;
    @FXML
    private Label startScreen;

    private ListView<Locale> languageList;
    private ListView<UIScale> uiScaleList;
    private ListView<StartScreen> startScreenList;

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
        initializeUiScales();
        initializeStartScreens();
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
        languageList = new ListView<>();

        languageList.getItems().addListener((InvalidationListener) observable -> languageList.setMaxHeight(50.0 * languageList.getItems().size()));
        languageList.getItems().addAll(UISettings.supportedLanguages());
        languageList.setCellFactory(param -> createLanguageCell());
        languageList.getSelectionModel().select(getUiSettings().getDefaultLanguage());
        languageList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var uiSettings = getUiSettings();
            uiSettings.setDefaultLanguage(newValue);
        });
    }

    private void initializeUiScales() {
        uiScaleList = new ListView<>();

        uiScaleList.getItems().addListener((InvalidationListener) observable -> uiScaleList.setMaxHeight(50.0 * uiScaleList.getItems().size()));
        uiScaleList.getItems().addAll(SettingsService.supportedUIScales());
        uiScaleList.getSelectionModel().select(getUiSettings().getUiScale());
        uiScaleList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var uiSettings = getUiSettings();
            uiSettings.setUiScale(newValue);
        });
    }

    private void initializeStartScreens() {
        startScreenList = new ListView<>();

        startScreenList.getItems().addListener((InvalidationListener) observable -> startScreenList.setMaxHeight(50.0 * startScreenList.getItems().size()));
        startScreenList.getItems().addAll(StartScreen.values());
        startScreenList.setCellFactory(param -> createStartScreenCell());
        startScreenList.getSelectionModel().select(getUiSettings().getStartScreen());
        startScreenList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var uiSettings = getUiSettings();
            uiSettings.setStartScreen(newValue);
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
        settingsSection.showOverlay(defaultLanguageCombo, languageList);
    }

    private void onUiScaleEvent() {
        settingsSection.showOverlay(uiScaleCombo, uiScaleList);
    }

    private void onStartScreenEvent() {
        settingsSection.showOverlay(startScreenCombo, startScreenList);
    }

    @FXML
    private void onDefaultLanguageKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onDefaultLanguageEvent();
        }
    }

    @FXML
    private void onUiScaleKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onUiScaleEvent();
        }
    }

    @FXML
    private void onStartScreenKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onStartScreenEvent();
        }
    }

    @FXML
    private void onDefaultLanguageClicked(MouseEvent event) {
        event.consume();
        onDefaultLanguageEvent();
    }

    @FXML
    private void onUiScaleClicked(MouseEvent event) {
        event.consume();
        onUiScaleEvent();
    }

    @FXML
    private void onStartScreenClicked(MouseEvent event) {
        event.consume();
        onStartScreenEvent();
    }

    //endregion
}

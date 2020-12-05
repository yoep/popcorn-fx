package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.ServerSettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.sections.SettingsSectionController;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.TextField;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URI;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class SettingsServerComponent extends AbstractSettingsUiComponent implements Initializable {
    private final SettingsSectionController settingsSection;

    @FXML
    private Pane apiServerPane;
    @FXML
    private Label apiServer;

    private TextField apiServerInput;

    public SettingsServerComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, SettingsService settingsService,
                                   SettingsSectionController settingsSection) {
        super(eventPublisher, localeText, settingsService);
        this.settingsSection = settingsSection;
    }

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeApiServer();
        initializeListeners();
    }

    private void initializeApiServer() {
        var settings = getSettings();

        // initialize the input field
        apiServerInput = new TextField();
        apiServerInput.setOnKeyPressed(event -> {
            if (event.getCode() == KeyCode.ENTER) {
                onApiServerChanged(apiServerInput.getText());
            }
        });
        apiServerInput.setPromptText("https://www.my-api.com");

        updateApiServer();
        settings.addListener(event -> {
            if (event.getPropertyName().equals(ServerSettings.API_SERVER_PROPERTY)) {
                updateApiServer();
            }
        });
    }

    private void initializeListeners() {
        settingsSection.addListener(() -> settingsSection.setBackspaceActionEnabled(true));
    }

    //endregion

    //region Functions

    private void onApiServerChanged(String newValue) {
        var settings = getSettings();

        try {
            if (StringUtils.isNotEmpty(newValue)) {
                settings.setApiServer(URI.create(newValue));
            } else {
                settings.setApiServer(null);
            }

            showNotification();
        } catch (IllegalArgumentException ex) {
            log.warn("API server is invalid, " + ex.getMessage(), ex);
        }
    }

    private void updateApiServer() {
        var settings = getSettings();
        var apiServerText = settings.getApiServer()
                .map(URI::toString)
                .orElse(null);

        apiServer.setText(apiServerText);
        apiServerInput.setText(apiServerText);
    }

    private void onApiServerEvent() {
        settingsSection.setBackspaceActionEnabled(false);
        settingsSection.showOverlay(apiServerPane, apiServerInput);
    }

    private ServerSettings getSettings() {
        return settingsService.getSettings().getServerSettings();
    }

    @FXML
    private void onApiServerKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onApiServerEvent();
        }
    }

    @FXML
    private void onApiServerClicked(MouseEvent event) {
        event.consume();
        onApiServerEvent();
    }

    //endregion
}

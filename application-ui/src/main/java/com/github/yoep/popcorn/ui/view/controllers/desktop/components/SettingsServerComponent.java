package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.ServerSettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import com.github.yoep.popcorn.ui.view.controls.DelayedTextField;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URI;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class SettingsServerComponent extends AbstractSettingsUiComponent implements Initializable {
    @FXML
    private DelayedTextField apiServer;

    //region Constructors

    public SettingsServerComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, SettingsService settingsService) {
        super(eventPublisher, localeText, settingsService);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeApiServer();
    }

    private void initializeApiServer() {
        var settings = getSettings();

        apiServer.setValue(settings.getApiServer()
                .map(URI::toString)
                .orElse(null));
        apiServer.valueProperty().addListener((observable, oldValue, newValue) -> onApiServerChanged(newValue));
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

    private ServerSettings getSettings() {
        return settingsService.getSettings().getServerSettings();
    }

    //endregion
}

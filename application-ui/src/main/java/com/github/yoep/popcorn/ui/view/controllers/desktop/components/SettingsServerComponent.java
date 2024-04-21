package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ServerSettings;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import com.github.yoep.popcorn.ui.view.controls.DelayedTextField;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class SettingsServerComponent extends AbstractSettingsUiComponent implements Initializable {
    @FXML
    private DelayedTextField apiServer;

    //region Constructors

    public SettingsServerComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig settingsService) {
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

        apiServer.setValue(settings.getApiServer());
        apiServer.valueProperty().addListener((observable, oldValue, newValue) -> onApiServerChanged(newValue));
    }

    //endregion

    //region Functions

    private void onApiServerChanged(String newValue) {
        var settings = getSettings();

        try {
            if (StringUtils.isNotEmpty(newValue)) {
                settings.setApiServer(newValue);
            } else {
                settings.setApiServer(null);
            }

            applicationConfig.update(settings);
            showNotification();
        } catch (IllegalArgumentException ex) {
            log.warn("API server is invalid, " + ex.getMessage(), ex);
        }
    }

    private ServerSettings getSettings() {
        return applicationConfig.getSettings().getServerSettings();
    }

    //endregion
}

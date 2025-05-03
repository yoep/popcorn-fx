package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import com.github.yoep.popcorn.ui.view.controls.DelayedTextField;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

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
        getSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                apiServer.setValue(settings.getApiServer());
                apiServer.valueProperty().addListener((observable, oldValue, newValue) -> onApiServerChanged(newValue));
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    //endregion

    //region Functions

    private void onApiServerChanged(String newValue) {
        getSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                applicationConfig.update(ApplicationSettings.ServerSettings.newBuilder(settings)
                        .setApiServer(newValue)
                        .build());
                showNotification();
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    private CompletableFuture<ApplicationSettings.ServerSettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getServerSettings);
    }

    //endregion
}

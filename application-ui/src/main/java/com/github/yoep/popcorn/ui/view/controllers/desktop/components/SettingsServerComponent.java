package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import com.github.yoep.popcorn.ui.view.controls.DelayedTextField;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.CheckBox;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Stream;

@Slf4j
public class SettingsServerComponent extends AbstractSettingsUiComponent implements Initializable {
    @FXML
    DelayedTextField moviesServers;
    @FXML
    DelayedTextField seriesServers;
    @FXML
    CheckBox updateServersAutomatically;

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
                moviesServers.setValue(listToString(settings.getMovieApiServersList()));
                moviesServers.valueProperty().addListener((observable, oldValue, newValue) -> onSettingsChanged());
                seriesServers.setValue(listToString(settings.getSerieApiServersList()));
                seriesServers.valueProperty().addListener((observable, oldValue, newValue) -> onSettingsChanged());
                updateServersAutomatically.setSelected(settings.getUpdateApiServersAutomatically());
                updateServersAutomatically.selectedProperty().addListener((observable, oldValue, newValue) -> onSettingsChanged());
            } else {
                log.error("Failed to retrieve settings", throwable);
                showErrorNotification(SettingsMessage.SETTINGS_FAILED_TO_LOAD);
            }
        });
    }

    //endregion

    //region Functions

    private void onSettingsChanged() {
        applicationConfig.update(createServerSettings());
        showNotification();
    }

    private ApplicationSettings.ServerSettings createServerSettings() {
        return ApplicationSettings.ServerSettings.newBuilder()
                .addAllMovieApiServers(stringToList(moviesServers.getValue()))
                .addAllSerieApiServers(stringToList(seriesServers.getValue()))
                .setUpdateApiServersAutomatically(updateServersAutomatically.isSelected())
                .build();
    }

    private CompletableFuture<ApplicationSettings.ServerSettings> getSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getServerSettings);
    }

    private String listToString(Iterable<String> list) {
        return String.join(",", list);
    }

    private List<String> stringToList(String string) {
        return Stream.of(string.split(","))
                .map(String::trim)
                .toList();
    }

    //endregion
}

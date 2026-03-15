package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsUiComponent;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.controls.VirtualKeyboard;
import javafx.animation.PauseTransition;
import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.stream.Stream;

@Slf4j
public class TvSettingsServerComponent extends AbstractSettingsUiComponent implements Initializable {
    final PauseTransition saveTimeout = new PauseTransition(Duration.seconds(5));

    @FXML
    Button movieServersBtn;
    @FXML
    Overlay movieServersOverlay;
    @FXML
    Label movieServersTxt;
    @FXML
    VirtualKeyboard movieServersInput;
    @FXML
    Button seriesServersBtn;
    @FXML
    Overlay serieServersOverlay;
    @FXML
    Label serieServersTxt;
    @FXML
    VirtualKeyboard serieServersInput;
    @FXML
    CheckBox updateServersAutomatically;

    public TvSettingsServerComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig settingsService) {
        super(eventPublisher, localeText, settingsService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeSettings();
        saveTimeout.setOnFinished(this::onSave);
    }

    private void initializeSettings() {
        applicationConfig.getSettings()
                .thenApply(ApplicationSettings::getServerSettings)
                .whenComplete((settings, throwable) -> {
                    if (throwable == null) {
                        Platform.runLater(() -> {
                            onSettingsLoaded(settings);
                            initializeListeners();
                        });
                    } else {
                        log.error("Failed to retrieve settings", throwable);
                        showErrorNotification(SettingsMessage.SETTINGS_FAILED_TO_LOAD);
                    }
                });
    }

    private void initializeListeners() {
        movieServersInput.textProperty().addListener((observable, oldValue, newValue) -> {
            movieServersBtn.setText(newValue);
            movieServersTxt.setText(newValue);
            saveTimeout.playFromStart();
        });
        serieServersInput.textProperty().addListener((observable, oldValue, newValue) -> {
            seriesServersBtn.setText(newValue);
            serieServersTxt.setText(newValue);
            saveTimeout.playFromStart();
        });
        updateServersAutomatically.selectedProperty().addListener((observable, oldValue, newValue) -> this.onSave(new ActionEvent()));
    }

    private void onSettingsLoaded(ApplicationSettings.ServerSettings settings) {
        var movieServers = listToString(settings.getMovieApiServersList());
        movieServersBtn.setText(movieServers);
        movieServersTxt.setText(movieServers);

        var serieServers = listToString(settings.getSerieApiServersList());
        seriesServersBtn.setText(serieServers);
        serieServersTxt.setText(serieServers);

        updateServersAutomatically.setSelected(settings.getUpdateApiServersAutomatically());
    }

    private void onSave(ActionEvent event) {
        event.consume();
        applicationConfig.update(createSettings());
        showNotification();
    }

    private ApplicationSettings.ServerSettings createSettings() {
        return ApplicationSettings.ServerSettings.newBuilder()
                .addAllMovieApiServers(stringToList(movieServersInput.getText()))
                .addAllSerieApiServers(stringToList(serieServersInput.getText()))
                .setUpdateApiServersAutomatically(updateServersAutomatically.isSelected())
                .build();
    }

    @FXML
    void onCloseInputOverlay(ActionEvent event) {
        event.consume();
        movieServersOverlay.hide();
        serieServersOverlay.hide();
    }

    private String listToString(Iterable<String> list) {
        return String.join(",", list);
    }

    private List<String> stringToList(String string) {
        return Stream.of(string.split(","))
                .map(String::trim)
                .toList();
    }
}

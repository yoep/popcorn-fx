package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ListCell;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class SettingsPlaybackComponent extends AbstractSettingsComponent implements Initializable {

    @FXML
    private ComboBox<ApplicationSettings.PlaybackSettings.Quality> quality;
    @FXML
    private CheckBox fullscreen;
    @FXML
    private CheckBox autoPlayNextEpisode;

    //region Constructors

    public SettingsPlaybackComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig settingsService) {
        super(eventPublisher, localeText, settingsService);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeQuality();
        initializeFullscreen();
        initializeAutoPlayNextEpisode();
    }

    private void initializeQuality() {
        var items = quality.getItems();

        items.add(null);
        items.addAll(ApplicationSettings.PlaybackSettings.Quality.values());

        getPlaybackSettings().thenAccept(settings -> Platform.runLater(() -> {
            quality.setCellFactory(param -> createQualityCell());
            quality.setButtonCell(createQualityCell());
            quality.getSelectionModel().select(Optional.ofNullable(settings.getQuality())
                    .filter(e -> settings.hasQuality())
                    .orElse(null));
            quality.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onQualityChanged(newValue));
        }));
    }

    private void initializeFullscreen() {
        getPlaybackSettings().thenAccept(settings -> Platform.runLater(() -> {
            fullscreen.setSelected(settings.getFullscreen());
            fullscreen.selectedProperty().addListener((observable, oldValue, newValue) -> onFullscreenChanged(newValue));
        }));
    }

    private void initializeAutoPlayNextEpisode() {
        getPlaybackSettings().thenAccept(settings -> Platform.runLater(() -> {
            autoPlayNextEpisode.setSelected(settings.getAutoPlayNextEpisodeEnabled());
            autoPlayNextEpisode.selectedProperty().addListener((observable, oldValue, newValue) -> onAutoPlayNextEpisodeChanged(newValue));
        }));
    }

    //endregion

    //region Functions

    private ListCell<ApplicationSettings.PlaybackSettings.Quality> createQualityCell() {
        return new ListCell<>() {
            @Override
            protected void updateItem(ApplicationSettings.PlaybackSettings.Quality item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    if (item != null) {
                        setText(item.toString());
                    } else {
                        setText("-");
                    }
                } else {
                    setText(null);
                }
            }
        };
    }

    void onQualityChanged(ApplicationSettings.PlaybackSettings.Quality newValue) {
        getPlaybackSettings().thenAccept(settings -> {
            applicationConfig.update(ApplicationSettings.PlaybackSettings.newBuilder(settings)
                    .setQuality(newValue)
                    .build());
            showNotification();
        });
    }

    void onFullscreenChanged(Boolean newValue) {
        getPlaybackSettings().thenAccept(settings -> {
            applicationConfig.update(ApplicationSettings.PlaybackSettings.newBuilder(settings)
                    .setFullscreen(newValue)
                    .build());
            showNotification();
        });
    }

    void onAutoPlayNextEpisodeChanged(Boolean newValue) {
        getPlaybackSettings().thenAccept(settings -> {
            applicationConfig.update(ApplicationSettings.PlaybackSettings.newBuilder(settings)
                    .setAutoPlayNextEpisodeEnabled(newValue)
                    .build());
            showNotification();
        });
    }

    private CompletableFuture<ApplicationSettings.PlaybackSettings> getPlaybackSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getPlaybackSettings);
    }

    //endregion
}

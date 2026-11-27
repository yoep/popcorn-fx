package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Arrays;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings.PlaybackSettings.Quality;

@Slf4j
public class TvSettingsPlaybackComponent extends AbstractSettingsComponent implements Initializable {
    static final String QUALITY_NONE_KEY = "settings_playback_quality_none";

    @FXML
    Button qualityButton;
    @FXML
    Overlay qualityOverlay;
    @FXML
    AxisItemSelection<Quality> qualities;
    @FXML
    CheckBox fullscreen;
    @FXML
    CheckBox autoPlayNextEpisode;

    public TvSettingsPlaybackComponent(EventPublisher eventPublisher, LocaleText localeText, ApplicationConfig applicationConfig) {
        super(eventPublisher, localeText, applicationConfig);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        qualities.setItemFactory(item -> new Button(qualityText(item)));
        // P0 is the "no default quality" entry, it clears the optional quality setting
        qualities.setItems(Arrays.stream(Quality.values())
                .filter(e -> e != Quality.UNRECOGNIZED)
                .toArray(Quality[]::new));

        getPlaybackSettings().thenAccept(settings -> Platform.runLater(() -> {
            var quality = settings.hasQuality() ? settings.getQuality() : Quality.P0;
            qualities.setSelectedItem(quality, true);
            qualityButton.setText(qualityText(quality));
            fullscreen.setSelected(settings.getFullscreen());
            autoPlayNextEpisode.setSelected(settings.getAutoPlayNextEpisodeEnabled());

            qualities.selectedItemProperty().addListener((observable, oldValue, newValue) -> onQualityChanged(newValue));
            fullscreen.selectedProperty().addListener((observable, oldValue, newValue) -> onFullscreenChanged(newValue));
            autoPlayNextEpisode.selectedProperty().addListener((observable, oldValue, newValue) -> onAutoPlayNextEpisodeChanged(newValue));
        }));
    }

    void onQualityChanged(Quality newValue) {
        if (newValue == null) {
            return;
        }

        getPlaybackSettings().thenAccept(settings -> {
            var builder = ApplicationSettings.PlaybackSettings.newBuilder(settings);
            if (newValue == Quality.P0) {
                builder.clearQuality();
            } else {
                builder.setQuality(newValue);
            }

            applicationConfig.update(builder.build());
            showNotification();
            Platform.runLater(() -> {
                qualityButton.setText(qualityText(newValue));
                qualityOverlay.hide();
            });
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

    private String qualityText(Quality quality) {
        return quality == Quality.P0 ? localeText.get(QUALITY_NONE_KEY) : quality.name().substring(1) + "p";
    }

    private CompletableFuture<ApplicationSettings.PlaybackSettings> getPlaybackSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getPlaybackSettings);
    }
}

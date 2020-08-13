package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ActivityManager;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.PlaybackSettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;

import java.net.URL;
import java.util.ResourceBundle;

public class SettingsPlaybackComponent extends AbstractSettingsComponent implements Initializable {

    @FXML
    private Label quality;

    //region Constructors

    public SettingsPlaybackComponent(ActivityManager activityManager, LocaleText localeText, SettingsService settingsService) {
        super(activityManager, localeText, settingsService);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeQuality();
    }

    private void initializeQuality() {
        var playbackSettings = getPlaybackSettings();

        updateQuality(playbackSettings.getQuality());
        playbackSettings.addListener(evt -> {
            if (evt.getPropertyName().equals(PlaybackSettings.QUALITY_PROPERTY)) {
                updateQuality((PlaybackSettings.Quality) evt.getNewValue());
            }
        });
    }

    //endregion

    //region Functions

    private void updateQuality(PlaybackSettings.Quality quality) {
        if (quality != null) {
            this.quality.setText(quality.toString());
        } else {
            this.quality.setText("-");
        }
    }

    private PlaybackSettings getPlaybackSettings() {
        return settingsService.getSettings().getPlaybackSettings();
    }

    //endregion
}

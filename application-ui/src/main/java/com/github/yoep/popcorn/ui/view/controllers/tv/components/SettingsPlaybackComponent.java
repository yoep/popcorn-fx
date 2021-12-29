package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.PlaybackSettings;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractSettingsComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.sections.SettingsSectionController;
import javafx.beans.InvalidationListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URL;
import java.util.ResourceBundle;

public class SettingsPlaybackComponent extends AbstractSettingsComponent implements Initializable {
    private final SettingsSectionController settingsSection;

    @FXML
    private Label quality;
    @FXML
    private Pane qualityCombo;
    @FXML
    private CheckBox autoPlayNextEpisode;

    private ListView<PlaybackSettings.Quality> qualityList;

    //region Constructors

    public SettingsPlaybackComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, SettingsService settingsService,
                                     SettingsSectionController settingsSection) {
        super(eventPublisher, localeText, settingsService);
        this.settingsSection = settingsSection;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeQuality();
        initializeAutoPlayNextEpisode();
    }

    private void initializeQuality() {
        var playbackSettings = getPlaybackSettings();

        updateQuality(playbackSettings.getQuality());
        playbackSettings.addListener(evt -> {
            if (evt.getPropertyName().equals(PlaybackSettings.QUALITY_PROPERTY)) {
                updateQuality((PlaybackSettings.Quality) evt.getNewValue());
            }
        });

        qualityList = new ListView<>();

        qualityList.getItems().addListener((InvalidationListener) observable -> qualityList.setMaxHeight(50.0 * qualityList.getItems().size()));
        qualityList.getItems().addAll(PlaybackSettings.Quality.values());
        qualityList.getSelectionModel().select(playbackSettings.getQuality());
        qualityList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            var settings = getPlaybackSettings();

            settings.setQuality(newValue);
            showNotification();
        });
    }

    private void initializeAutoPlayNextEpisode() {
        var playbackSettings = getPlaybackSettings();

        autoPlayNextEpisode.setSelected(playbackSettings.isAutoPlayNextEpisodeEnabled());
        autoPlayNextEpisode.selectedProperty().addListener((observable, oldValue, newValue) -> onAutoPlayNextEpisodeChanged(newValue));
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

    private void onQualityEvent() {
        settingsSection.showOverlay(qualityCombo, qualityList);
    }

    private void onAutoPlayNextEpisodeChanged(Boolean newValue) {
        var settings = getPlaybackSettings();

        settings.setAutoPlayNextEpisodeEnabled(newValue);
        showNotification();
    }

    private PlaybackSettings getPlaybackSettings() {
        return settingsService.getSettings().getPlaybackSettings();
    }

    @FXML
    private void onQualityKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onQualityEvent();
        }
    }

    @FXML
    private void onQualityClicked(MouseEvent event) {
        event.consume();
        onQualityEvent();
    }

    //endregion
}

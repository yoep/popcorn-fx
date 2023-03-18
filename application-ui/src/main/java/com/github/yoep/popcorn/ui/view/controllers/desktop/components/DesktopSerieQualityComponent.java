package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.events.MediaQualityChangedEvent;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.services.VideoQualityService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
@RequiredArgsConstructor
public class DesktopSerieQualityComponent implements Initializable {
    private final EventPublisher eventPublisher;
    private final VideoQualityService videoQualityService;

    private Episode media;

    @FXML
    AxisItemSelection<String> qualities;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeQualities();
    }

    public String getSelectedQuality() {
        return qualities.getSelectedItem();
    }

    public void episodeChanged(Episode episode) {
        this.media = episode;
        var resolutions = Optional.ofNullable(this.media)
                .map(Episode::getTorrents)
                .map(videoQualityService::getVideoResolutions)
                .orElse(new String[0]);
        var defaultResolution = videoQualityService.getDefaultVideoResolution(asList(resolutions));

        Platform.runLater(() -> {
            qualities.setItems(resolutions);
            qualities.setSelectedItem(defaultResolution);
        });
    }

    private void initializeQualities() {
        qualities.setItemFactory(item -> {
            var node = new Label(item);
            node.getStyleClass().add("quality");
            return node;
        });
        qualities.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                eventPublisher.publish(new MediaQualityChangedEvent(this, media, newValue));
            }
        });
    }
}

package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.media.MovieDetails;
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
import java.util.Objects;
import java.util.Optional;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
@RequiredArgsConstructor
public class DesktopMovieQualityComponent implements Initializable {
    static final String DEFAULT_TORRENT_AUDIO = "en";

    private final EventPublisher eventPublisher;
    private final VideoQualityService videoQualityService;

    private MovieDetails media;

    @FXML
    AxisItemSelection<String> qualities;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeQualities();
        eventPublisher.register(ShowMovieDetailsEvent.class, event -> {
            onShowDetails(event);
            return event;
        });
    }

    public String getSelectedQuality() {
        return qualities.getSelectedItem();
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

    private void onShowDetails(ShowMovieDetailsEvent event) {
        this.media = event.getMedia();
        var resolutions = Optional.ofNullable(this.media)
                .map(MovieDetails::getTorrents)
                .flatMap(e -> e.stream()
                        .filter(language -> Objects.equals(language.getLanguage(), DEFAULT_TORRENT_AUDIO))
                        .findFirst())
                .map(Media.TorrentLanguage::getTorrents)
                .map(videoQualityService::getVideoResolutions)
                .orElse(new String[0]);
        videoQualityService.getDefaultVideoResolution(asList(resolutions)).whenComplete((defaultResolution, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> {
                    qualities.setItems(resolutions);
                    qualities.setSelectedItem(defaultResolution);
                });
            } else {
                log.error("Failed to retrieve video resolution", throwable);
            }
        });
    }
}

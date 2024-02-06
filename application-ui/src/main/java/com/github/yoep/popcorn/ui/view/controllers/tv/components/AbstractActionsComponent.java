package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.models.*;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.services.VideoQualityService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
@RequiredArgsConstructor
public abstract class AbstractActionsComponent implements Initializable {
    protected final EventPublisher eventPublisher;
    protected final SubtitleService subtitleService;
    protected final VideoQualityService videoQualityService;
    protected final PlaylistManager playlistManager;

    private CompletableFuture<List<SubtitleInfo>> subtitleFuture;

    @FXML
    Overlay qualityOverlay;
    @FXML
    AxisItemSelection<String> qualities;
    @FXML
    Overlay subtitleOverlay;
    @FXML
    AxisItemSelection<SubtitleInfo> subtitles;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        eventPublisher.register(ShowDetailsEvent.class, event -> {
            Platform.runLater(() -> {
                qualityOverlay.hide();
                subtitleOverlay.hide();
            });
            return event;
        });
        qualities.setOnItemActivated(newValue -> {
            qualityOverlay.hide();
            subtitleOverlay.show();
            subtitles.setSelectedItem(subtitleService.getDefaultOrInterfaceLanguage(subtitles.getItems()), true);
        });
        subtitles.setOnItemActivated(subtitle -> {
            subtitleOverlay.hide();
            subtitleService.updateSubtitle(subtitle);
            play();
        });
        subtitles.setItemFactory(item -> new Button(item.getLanguage().getNativeName()));
    }

    /**
     * Retrieve the currently displayed media item.
     */
    protected abstract Media getMedia();

    /**
     * Retrieve the sub item of the media.
     */
    protected abstract Media getSubItem();

    /**
     * Retrieve the available torrents for the current media.
     */
    protected abstract Map<String, MediaTorrentInfo> getTorrents();

    /**
     * Retrieve the subtitle for the current media item.
     */
    protected abstract CompletableFuture<List<SubtitleInfo>> retrieveSubtitles();

    protected void updateQualities() {
        qualities.setItems(videoQualityService.getVideoResolutions(getTorrents()));
        updateSubtitles();
    }

    private void play() {
        if (getMedia() instanceof MovieDetails movie) {
            playlistManager.play(movie, qualities.getSelectedItem());
        } else if (getMedia() instanceof ShowDetails show) {
            var episode = (Episode) getSubItem();
            playlistManager.play(show, episode, qualities.getSelectedItem());
        } else {
            log.warn("Unable to start playback, unsupported media type {}", getMedia().getClass().getSimpleName());
        }
    }

    private void updateSubtitles() {
        if (subtitleFuture != null)
            subtitleFuture.cancel(true);

        subtitles.setItems(subtitleService.none());
        subtitleFuture = retrieveSubtitles()
                .whenComplete((subtitleInfos, throwable) -> {
                    if (throwable == null) {
                        Platform.runLater(() -> subtitles.setItems(subtitleInfos.toArray(new SubtitleInfo[0])));
                    } else {
                        log.error(throwable.getMessage(), throwable);
                    }
                });
    }
}

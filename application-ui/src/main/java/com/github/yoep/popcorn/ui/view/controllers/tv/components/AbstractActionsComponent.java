package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.view.ViewHelper;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
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
        qualities.setOnItemActivated(newValue -> {
            qualityOverlay.hide();
            subtitleOverlay.show();
            subtitles.setSelectedItem(subtitleService.getDefaultOrInterfaceLanguage(subtitles.getItems()), true);
        });
        subtitles.setOnItemActivated(subtitle -> {
            subtitleOverlay.hide();

            var quality = qualities.getSelectedItem();
            var mediaTorrentInfo = getTorrents().get(quality);
            eventPublisher.publishEvent(new LoadMediaTorrentEvent(this, mediaTorrentInfo, getMedia(), getSubItem(), quality, subtitle));
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
        qualities.setItems(ViewHelper.getVideoResolutions(getTorrents()));
        updateSubtitles();
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

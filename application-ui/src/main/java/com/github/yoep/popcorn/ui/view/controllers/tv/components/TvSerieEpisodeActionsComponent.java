package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controllers.common.components.SerieActionsComponent;
import com.github.yoep.popcorn.ui.view.services.VideoQualityService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class TvSerieEpisodeActionsComponent extends AbstractActionsComponent implements SerieActionsComponent {
    private ShowDetails media;
    private Episode episode;

    private Runnable eventHandler;

    @FXML
    Button watchNowButton;

    public TvSerieEpisodeActionsComponent(EventPublisher eventPublisher, SubtitleService subtitleService, VideoQualityService videoQualityService) {
        super(eventPublisher, subtitleService, videoQualityService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        qualityOverlay.shownProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                eventHandler.run();
                Platform.runLater(() -> qualities.setSelectedItem(videoQualityService.getDefaultVideoResolution(qualities.getItems()), true));
            }
        });
    }

    @Override
    public void episodeChanged(ShowDetails media, Episode episode) {
        this.media = media;
        this.episode = episode;

        updateQualities();
        watchNowButton.requestFocus();
    }

    @Override
    public void setOnWatchNowClicked(Runnable eventHandler) {
        this.eventHandler = eventHandler;
    }

    @Override
    protected Media getMedia() {
        return media;
    }

    @Override
    protected Media getSubItem() {
        return episode;
    }

    @Override
    protected Map<String, MediaTorrentInfo> getTorrents() {
        return episode.getTorrents();
    }

    @Override
    protected CompletableFuture<List<SubtitleInfo>> retrieveSubtitles() {
        return subtitleService.retrieveSubtitles(media, episode);
    }
}

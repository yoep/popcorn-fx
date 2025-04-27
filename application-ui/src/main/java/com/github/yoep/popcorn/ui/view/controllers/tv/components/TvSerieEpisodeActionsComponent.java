package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.playlists.DefaultPlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.ui.view.controllers.common.components.SerieActionsComponent;
import com.github.yoep.popcorn.ui.view.services.VideoQualityService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class TvSerieEpisodeActionsComponent extends AbstractActionsComponent implements SerieActionsComponent {
    private ShowDetails media;
    private Episode episode;

    private Runnable eventHandler;

    @FXML
    Button watchNowButton;

    public TvSerieEpisodeActionsComponent(EventPublisher eventPublisher, ISubtitleService subtitleService, VideoQualityService videoQualityService, DefaultPlaylistManager playlistManager) {
        super(eventPublisher, subtitleService, videoQualityService, playlistManager);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        qualityOverlay.shownProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                eventHandler.run();
                videoQualityService.getDefaultVideoResolution(qualities.getItems()).whenComplete((quality, throwable) -> {
                    if (throwable == null) {
                        Platform.runLater(() -> qualities.setSelectedItem(quality, true));
                    } else {
                        log.error("Failed to retrieve video resolution", throwable);
                    }
                });
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
    protected Optional<com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.TorrentQuality> getTorrents() {
        return Optional.ofNullable(episode.getTorrents());
    }

    @Override
    protected CompletableFuture<List<ISubtitleInfo>> retrieveSubtitles() {
        return subtitleService.retrieveSubtitles(media, episode);
    }
}

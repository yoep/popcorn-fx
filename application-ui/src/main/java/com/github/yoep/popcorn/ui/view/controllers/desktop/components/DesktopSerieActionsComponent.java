package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.utils.WatchNowUtils;
import com.github.yoep.popcorn.ui.view.controllers.components.SerieActionsComponent;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
@RequiredArgsConstructor
public class DesktopSerieActionsComponent implements Initializable, SerieActionsComponent {
    private final EventPublisher eventPublisher;
    private final PlayerManagerService playerManagerService;
    private final SubtitleService subtitleService;
    private final DesktopSerieQualityComponent desktopSerieQualityComponent;

    private ShowDetails media;
    private Episode episode;
    private CompletableFuture<List<SubtitleInfo>> subtitlesFuture;

    @FXML
    PlayerDropDownButton watchNowButton;
    @FXML
    LanguageFlagSelection languageSelection;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        WatchNowUtils.syncPlayerManagerAndWatchNowButton(playerManagerService, watchNowButton);

        initializeLanguage();
    }

    private void initializeLanguage() {
        languageSelection.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null && newValue.isNone()) {
                subtitleService.disableSubtitle();
            } else {
                subtitleService.updateSubtitle(newValue);
            }
        });
    }

    @Override
    public void episodeChanged(ShowDetails media, Episode episode) {
        this.media = media;
        this.episode = episode;

        desktopSerieQualityComponent.episodeChanged(episode);
        updateSubtitles();
    }

    private void updateSubtitles() {
        if (subtitlesFuture != null) {
            subtitlesFuture.cancel(true);
        }

        var languages = languageSelection.getItems();
        languages.clear();
        languages.setAll(subtitleService.none(), subtitleService.custom());
        subtitlesFuture = subtitleService.retrieveSubtitles(media, episode)
                .whenComplete((subtitleInfos, throwable) -> {
                    if (throwable == null) {
                        Platform.runLater(() -> {
                            languages.clear();
                            languages.setAll(subtitleInfos.toArray(SubtitleInfo[]::new));
                            languageSelection.setSelectedItem(subtitleService.getDefaultOrInterfaceLanguage(subtitleInfos));
                        });
                    } else {
                        log.error(throwable.getMessage(), throwable);
                    }
                });
    }

    @Override
    public void setOnWatchNowClicked(Runnable eventHandler) {

    }

    @FXML
    void onWatchNowClicked() {
        var mediaTorrentInfo = episode.getTorrents().get(desktopSerieQualityComponent.getSelectedQuality());
        eventPublisher.publishEvent(new LoadMediaTorrentEvent(this, mediaTorrentInfo, media, episode, desktopSerieQualityComponent.getSelectedQuality(),
                languageSelection.getSelectedItem()));
    }
}

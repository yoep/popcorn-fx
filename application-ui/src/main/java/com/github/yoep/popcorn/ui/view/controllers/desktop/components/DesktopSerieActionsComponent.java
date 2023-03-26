package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.utils.WatchNowUtils;
import com.github.yoep.popcorn.ui.view.controllers.common.components.SerieActionsComponent;
import com.github.yoep.popcorn.ui.view.controls.LanguageFlagCell;
import com.github.yoep.popcorn.ui.view.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
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

    @Override
    public void episodeChanged(ShowDetails media, Episode episode) {
        this.media = media;
        this.episode = episode;

        desktopSerieQualityComponent.episodeChanged(episode);
        updateSubtitles();
    }

    @Override
    public void setOnWatchNowClicked(Runnable eventHandler) {
        // no-op
    }

    private void initializeLanguage() {
        languageSelection.selectedItemProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null && newValue.isNone()) {
                subtitleService.disableSubtitle();
            }
        });
        languageSelection.setFactory(new LanguageFlagCell() {
            @Override
            public void updateItem(SubtitleInfo item) {
                if (item == null)
                    return;

                setText(item.getLanguage().getNativeName());
                try {
                    var image = new ImageView(new Image(item.getFlagResource().getInputStream()));

                    image.setFitHeight(15);
                    image.setPreserveRatio(true);

                    setGraphic(image);
                } catch (IOException ex) {
                    log.error(ex.getMessage(), ex);
                }
            }
        });
    }

    private void updateSubtitles() {
        if (subtitlesFuture != null) {
            subtitlesFuture.cancel(true);
        }

        var languages = languageSelection.getItems();
        var defaultSubtitle = subtitleService.none();
        languages.clear();
        languages.setAll(defaultSubtitle, subtitleService.custom());
        languageSelection.select(defaultSubtitle);
        subtitlesFuture = subtitleService.retrieveSubtitles(media, episode)
                .whenComplete((subtitleInfos, throwable) -> {
                    if (throwable == null) {
                        Platform.runLater(() -> {
                            languageSelection.getItems().clear();
                            languageSelection.getItems().setAll(subtitleInfos.toArray(SubtitleInfo[]::new));
                            languageSelection.select(subtitleService.getDefaultOrInterfaceLanguage(subtitleInfos));
                        });
                    } else {
                        log.error(throwable.getMessage(), throwable);
                    }
                });
    }

    private void startMoviePlayback() {
        var mediaTorrentInfo = episode.getTorrents().get(desktopSerieQualityComponent.getSelectedQuality());
        eventPublisher.publishEvent(new LoadMediaTorrentEvent(this, mediaTorrentInfo, media, episode, desktopSerieQualityComponent.getSelectedQuality(),
                languageSelection.getSelectedItem()));
    }

    @FXML
    void onWatchNowClicked(MouseEvent event) {
        event.consume();
        startMoviePlayback();
    }

    @FXML
    void onWatchNowPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            startMoviePlayback();
        }
    }
}

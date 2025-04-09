package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.utils.WatchNowUtils;
import com.github.yoep.popcorn.ui.view.controllers.common.components.SerieActionsComponent;
import com.github.yoep.popcorn.ui.view.controls.LanguageFlagCell;
import com.github.yoep.popcorn.ui.view.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
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

import java.net.URL;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
@RequiredArgsConstructor
public class DesktopSerieActionsComponent implements Initializable, SerieActionsComponent {
    private final PlayerManagerService playerManagerService;
    private final SubtitleService subtitleService;
    private final DesktopSerieQualityComponent desktopSerieQualityComponent;
    private final PlaylistManager playlistManager;
    private final DetailsComponentService detailsComponentService;

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
            if (newValue.getLanguage() == Subtitle.Language.CUSTOM) {
                detailsComponentService.onCustomSubtitleSelected(() ->
                        subtitleService.none().whenComplete((language, throwable) -> {
                            if (throwable == null) {
                                languageSelection.select(language);
                            } else {
                                log.error("Failed to load none subtitle", throwable);
                            }
                        }));
            } else if (newValue.getLanguage() == Subtitle.Language.NONE) {
                subtitleService.disableSubtitle();
            } else {
                subtitleService.updateSubtitle(newValue);
            }
        });
        languageSelection.setFactory(new LanguageFlagCell() {
            @Override
            public void updateItem(Subtitle.Info item) {
                if (item == null)
                    return;

                setText(SubtitleHelper.getNativeName(item.getLanguage()));
//                var image = new ImageView(Optional.ofNullable(item.getFlagResource())
//                        .map(DesktopSerieActionsComponent.class::getResourceAsStream)
//                        .map(Image::new)
//                        .orElse(null));

//                image.setFitHeight(15);
//                image.setPreserveRatio(true);
//
//                setGraphic(image);
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
//        languages.setAll(defaultSubtitle, subtitleService.custom());
//        languageSelection.select(defaultSubtitle);
//        subtitlesFuture = subtitleService.retrieveSubtitles(media, episode)
//                .whenComplete((subtitleInfos, throwable) -> {
//                    if (throwable == null) {
//                        Platform.runLater(() -> {
//                            languageSelection.getItems().clear();
//                            languageSelection.getItems().setAll(subtitleInfos.toArray(SubtitleInfo[]::new));
//                            languageSelection.select(subtitleService.getDefaultOrInterfaceLanguage(subtitleInfos));
//                        });
//                    } else {
//                        log.error(throwable.getMessage(), throwable);
//                    }
//                });
    }

    private void startSeriePlayback() {
        playlistManager.play(media, episode, desktopSerieQualityComponent.getSelectedQuality());
    }

    @FXML
    void onWatchNowClicked(MouseEvent event) {
        event.consume();
        startSeriePlayback();
    }

    @FXML
    void onWatchNowPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            startSeriePlayback();
        }
    }
}

package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.messages.SubtitleMessage;
import com.github.yoep.popcorn.backend.playlists.model.Playlist;
import com.github.yoep.popcorn.backend.playlists.model.PlaylistItem;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.listeners.LanguageSelectionListener;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.utils.WatchNowUtils;
import com.github.yoep.popcorn.ui.view.ViewHelper;
import com.github.yoep.popcorn.ui.view.controls.LanguageFlagCell;
import com.github.yoep.popcorn.ui.view.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Tooltip;
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
public class DesktopMovieActionsComponent implements Initializable {
    private final PlayerManagerService playerService;
    private final PlaylistManager playlistManager;
    private final EventPublisher eventPublisher;
    private final LocaleText localeText;
    private final SubtitleService subtitleService;
    private final DetailsComponentService detailsComponentService;
    private final DesktopMovieQualityComponent desktopMovieQualityComponent;

    private MovieDetails media;
    private CompletableFuture<List<SubtitleInfo>> subtitleFuture;

    @FXML
    PlayerDropDownButton watchNowButton;
    @FXML
    Button watchTrailerButton;
    @FXML
    LanguageFlagSelection languageSelection;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        WatchNowUtils.syncPlayerManagerAndWatchNowButton(playerService, watchNowButton);
        initializeListeners();
        initializeLanguageSelection();
    }

    private void initializeListeners() {
        eventPublisher.register(ShowMovieDetailsEvent.class, event -> {
            this.media = event.getMedia();
            onShowMovieDetails();
            return event;
        });
    }

    private void initializeLanguageSelection() {
        languageSelection.setFactory(new LanguageFlagCell() {
            @Override
            public void updateItem(SubtitleInfo item) {
                if (item == null)
                    return;

                setText(null);

                var language = item.language().getNativeName();
                var image = Optional.ofNullable(item.getFlagResource())
                        .map(DesktopMovieActionsComponent.class::getResourceAsStream)
                        .map(Image::new)
                        .map(ImageView::new)
                        .orElseGet(ImageView::new);

                image.setFitHeight(20);
                image.setPreserveRatio(true);

                if (item.isNone()) {
                    language = localeText.get(SubtitleMessage.NONE);
                } else if (item.isCustom()) {
                    language = localeText.get(SubtitleMessage.CUSTOM);
                }

                var tooltip = new Tooltip(language);

                ViewHelper.instantTooltip(tooltip);
                Tooltip.install(image, tooltip);

                setGraphic(image);
            }
        });

        resetLanguageSelection();
        languageSelection.addListener(createLanguageListener());
    }

    private void resetLanguageSelection() {
        languageSelection.getItems().clear();
        languageSelection.getItems().add(subtitleService.none());
        languageSelection.select(0);
    }

    private void onShowMovieDetails() {
        var trailer = media.getTrailer();

        Platform.runLater(() -> {
            watchNowButton.select(playerService.getActivePlayer().orElse(null));
            watchTrailerButton.setVisible(trailer != null && !trailer.isBlank());
            watchNowButton.requestFocus();

            updateSubtitles();
        });
    }

    private void updateSubtitles() {
        if (subtitleFuture != null) {
            subtitleFuture.cancel(true);
        }

        var items = languageSelection.getItems();
        var defaultSubtitle = subtitleService.none();

        items.clear();
        items.addAll(defaultSubtitle, subtitleService.custom());
        languageSelection.select(defaultSubtitle);
        subtitleFuture = subtitleService
                .retrieveSubtitles(media)
                .whenComplete((subtitleInfos, throwable) -> {
                    if (throwable == null) {
                        Platform.runLater(() -> {
                            languageSelection.getItems().clear();
                            languageSelection.getItems().addAll(subtitleInfos.toArray(SubtitleInfo[]::new));
                            languageSelection.select(subtitleService.getDefaultOrInterfaceLanguage(subtitleInfos));
                        });
                    } else {
                        log.error(throwable.getMessage(), throwable);
                    }
                });
    }

    private void onWatchNow() {
        playlistManager.play(media, desktopMovieQualityComponent.getSelectedQuality());
    }

    private void playTrailer() {
        var item = PlaylistItem.builder()
                .url(media.getTrailer())
                .title(media.getTitle())
                .caption("Trailer")
                .thumb(media.getImages().getPoster())
                .subtitlesEnabled(false)
                .build();
        playlistManager.play(new Playlist(item));
    }

    protected LanguageSelectionListener createLanguageListener() {
        return newValue -> {
            if (newValue.isCustom()) {
                detailsComponentService.onCustomSubtitleSelected(() ->
                        languageSelection.select(subtitleService.none()));
            } else if (newValue.isNone()) {
                subtitleService.disableSubtitle();
            } else {
                subtitleService.updateSubtitle(newValue);
            }
        };
    }

    @FXML
    void onWatchNowClicked(MouseEvent event) {
        event.consume();
        onWatchNow();
    }

    @FXML
    void onWatchNowPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onWatchNow();
        }
    }

    @FXML
    void onTrailerClicked(MouseEvent event) {
        event.consume();
        playTrailer();
    }

    @FXML
    void onTrailerPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            playTrailer();
        }
    }

    @FXML
    void onSubtitleLabelClicked(MouseEvent event) {
        event.consume();
        languageSelection.show();
    }
}

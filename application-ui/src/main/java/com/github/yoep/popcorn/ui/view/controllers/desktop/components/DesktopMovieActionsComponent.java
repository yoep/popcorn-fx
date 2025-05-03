package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.media.MediaHelper;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.messages.SubtitleMessage;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.subtitles.LanguageSelectionListener;
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
    private final ISubtitleService subtitleService;
    private final DetailsComponentService detailsComponentService;
    private final DesktopMovieQualityComponent desktopMovieQualityComponent;

    private MovieDetails media;
    private CompletableFuture<List<ISubtitleInfo>> subtitleFuture;

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
            public void updateItem(ISubtitleInfo item) {
                if (item == null)
                    return;

                setText(null);

                var language = SubtitleHelper.getNativeName(item.getLanguage());
                var image = Optional.of(SubtitleHelper.getFlagResource(item.getLanguage()))
                        .map(DesktopMovieActionsComponent.class::getResourceAsStream)
                        .map(Image::new)
                        .map(ImageView::new)
                        .orElseGet(ImageView::new);

                image.setFitHeight(20);
                image.setPreserveRatio(true);

                if (item.getLanguage() == Subtitle.Language.NONE) {
                    language = localeText.get(SubtitleMessage.NONE);
                } else if (item.getLanguage() == Subtitle.Language.CUSTOM) {
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
        subtitleService.defaultSubtitles().whenComplete((subtitles, throwable) -> {
            if (throwable == null) {
                languageSelection.getItems().clear();
                languageSelection.getItems().addAll(subtitles);
                languageSelection.select(subtitles.getFirst());
            } else {
                log.error("Failed to load none subtitle", throwable);
            }
        });
    }

    private void onShowMovieDetails() {
        var trailer = media.proto().getTrailer();
        Platform.runLater(() -> {
            watchTrailerButton.setVisible(trailer != null && !trailer.isBlank());
            watchNowButton.requestFocus();
        });

        updateSubtitles();
    }

    private void updateSubtitles() {
        if (subtitleFuture != null) {
            subtitleFuture.cancel(true);
        }

        var items = languageSelection.getItems();

        subtitleService.defaultSubtitles().whenComplete((defaultSubtitles, throwable) -> {
            items.clear();
            items.addAll(defaultSubtitles);
            languageSelection.select(defaultSubtitles.getFirst());
        });

        subtitleFuture = subtitleService
                .retrieveSubtitles(media)
                .whenComplete((subtitles, throwable) -> {
                    if (throwable == null) {
                        Platform.runLater(() -> {
                            languageSelection.getItems().clear();
                            languageSelection.getItems().addAll(subtitles);
                        });

                        subtitleService.getDefaultOrInterfaceLanguage(subtitles).whenComplete((subtitle, ex) -> {
                            if (ex == null) {
                                languageSelection.select(subtitle);
                            } else {
                                log.error("Failed to retrieve default subtitle", ex);
                            }
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
        playlistManager.play(Playlist.newBuilder()
                .addItems(Playlist.Item.newBuilder()
                        .setUrl(media.proto().getTrailer())
                        .setTitle(media.title())
                        .setCaption("Trailer")
                        .setThumb(media.images().getPoster())
                        .setMedia(MediaHelper.getItem(media))
                        .setSubtitlesEnabled(false)
                        .build())
                .build());
    }

    protected LanguageSelectionListener createLanguageListener() {
        return newValue -> {
            var language = newValue.getLanguage();
            if (language == Subtitle.Language.CUSTOM) {
                detailsComponentService.onCustomSubtitleSelected(() ->
                        subtitleService.defaultSubtitles().whenComplete((subtitles, throwable) -> {
                            if (throwable == null) {
                                languageSelection.select(subtitles.getFirst());
                            } else {
                                log.error("Failed to load default subtitles", throwable);
                            }
                        }));
            } else if (language == Subtitle.Language.NONE) {
                subtitleService.disableSubtitle();
            } else {
                if (newValue instanceof SubtitleInfoWrapper(Subtitle.Info proto)) {
                    subtitleService.updatePreferredLanguage(proto.getLanguage());
                }
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

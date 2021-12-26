package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.messages.DetailsMessage;
import com.github.yoep.popcorn.backend.messages.SubtitleMessage;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.subtitles.controls.LanguageFlagCell;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;

import java.io.IOException;
import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class MovieDetailsComponent extends AbstractDesktopDetailsComponent<Movie> {
    private static final String DEFAULT_TORRENT_AUDIO = "en";

    @FXML
    private Label title;
    @FXML
    private Label overview;
    @FXML
    private Label year;
    @FXML
    private Label duration;
    @FXML
    private Label genres;
    @FXML
    private Tooltip watchedTooltip;
    @FXML
    private Tooltip favoriteTooltip;
    @FXML
    private Button watchTrailerButton;

    //region Constructors

    public MovieDetailsComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, HealthService healthService,
                                 SubtitleService subtitleService, SubtitlePickerService subtitlePickerService, ImageService imageService,
                                 SettingsService settingsService, FavoriteService favoriteService, WatchedService watchedService,
                                 PlayerManagerService playerService) {
        super(eventPublisher, localeText, healthService, subtitleService, subtitlePickerService, imageService, settingsService, favoriteService,
                watchedService, playerService);

    }

    //endregion

    //region Methods

    @EventListener
    public void onShowMovieDetails(ShowMovieDetailsEvent event) {
        Platform.runLater(() -> load(event.getMedia()));
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeTooltips();
        initializeLanguageSelection();
        initializeWatchNow();
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void load(Movie media) {
        super.load(media);

        loadText();
        loadButtons();
        loadSubtitles();
        loadFavoriteAndWatched();
        loadQualitySelection(media.getTorrents().get(DEFAULT_TORRENT_AUDIO));
    }

    @Override
    protected CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return imageService.loadPoster(media);
    }

    @Override
    protected void reset() {
        super.reset();
        resetLanguageSelection();

        title.setText(StringUtils.EMPTY);
        overview.setText(StringUtils.EMPTY);
        year.setText(StringUtils.EMPTY);
        duration.setText(StringUtils.EMPTY);
        genres.setText(StringUtils.EMPTY);
        favoriteIcon.getStyleClass().remove(LIKED_STYLE_CLASS);
        watchedIcon.getStyleClass().remove(WATCHED_STYLE_CLASS);
        qualitySelectionPane.getChildren().clear();
        poster.setImage(null);
    }

    //endregion

    //region Functions

    private void initializeTooltips() {
        var tooltip = new Tooltip(localeText.get(DetailsMessage.MAGNET_LINK));
        instantTooltip(tooltip);
        Tooltip.install(magnetLink, tooltip);

        instantTooltip(watchedTooltip);
        instantTooltip(favoriteTooltip);
    }

    private void initializeLanguageSelection() {
        languageSelection.setFactory(new LanguageFlagCell() {
            @Override
            public void updateItem(SubtitleInfo item) {
                if (item == null)
                    return;

                setText(null);

                try {
                    var language = item.getLanguage().getNativeName();
                    var image = new ImageView(new Image(item.getFlagResource().getInputStream()));

                    image.setFitHeight(20);
                    image.setPreserveRatio(true);

                    if (item.isNone()) {
                        language = localeText.get(SubtitleMessage.NONE);
                    } else if (item.isCustom()) {
                        language = localeText.get(SubtitleMessage.CUSTOM);
                    }

                    var tooltip = new Tooltip(language);

                    instantTooltip(tooltip);
                    Tooltip.install(image, tooltip);

                    setGraphic(image);
                } catch (IOException ex) {
                    log.error(ex.getMessage(), ex);
                }
            }
        });

        languageSelection.addListener(createLanguageListener());
        resetLanguageSelection();
    }

    private void loadText() {
        title.setText(media.getTitle());
        overview.setText(media.getSynopsis());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        genres.setText(String.join(" / ", media.getGenres()));
    }

    private void loadButtons() {
        watchTrailerButton.setVisible(StringUtils.isNotEmpty(media.getTrailer()));
        watchNowButton.select(playerService.getActivePlayer().orElse(null));
    }

    private void loadSubtitles() {
        resetLanguageSelection();
        languageSelection.setLoading(true);
        subtitleService.retrieveSubtitles(media).whenComplete(this::handleSubtitlesResponse);
    }

    @Override
    protected void switchActiveQuality(String quality) {
        super.switchActiveQuality(quality);
        switchHealth(media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality));
    }

    @Override
    protected void switchWatched(boolean isWatched) {
        super.switchWatched(isWatched);

        if (isWatched) {
            watchedTooltip.setText(localeText.get(DetailsMessage.MARK_AS_NOT_SEEN));
        } else {
            watchedTooltip.setText(localeText.get(DetailsMessage.MARK_AS_SEEN));
        }
    }

    @Override
    protected void switchLiked(boolean isLiked) {
        super.switchLiked(isLiked);

        if (isLiked) {
            favoriteTooltip.setText(localeText.get(DetailsMessage.REMOVE_FROM_BOOKMARKS));
        } else {
            favoriteTooltip.setText(localeText.get(DetailsMessage.ADD_TO_BOOKMARKS));
        }
    }

    @FXML
    private void onMagnetClicked(MouseEvent event) {
        event.consume();
        var torrentInfo = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);

        if (event.getButton() == MouseButton.SECONDARY) {
            copyMagnetLink(torrentInfo);
        } else {
            openMagnetLink(torrentInfo);
        }
    }

    @FXML
    private void onWatchNowClicked(MouseEvent event) {
        event.consume();

        var mediaTorrentInfo = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);
        eventPublisher.publishEvent(new LoadMediaTorrentEvent(this, mediaTorrentInfo, media, quality, subtitle));
    }

    @FXML
    private void onTrailerClicked(MouseEvent event) {
        event.consume();

        eventPublisher.publishEvent(new PlayVideoEvent(this, media.getTrailer(), media.getTitle(), false, media.getImages().getFanart()));
    }

    @FXML
    private void onSubtitleLabelClicked(MouseEvent event) {
        event.consume();
        languageSelection.show();
    }

    @FXML
    private void onFavoriteClicked(MouseEvent event) {
        event.consume();
        if (!media.isLiked()) {
            favoriteService.addToFavorites(media);
        } else {
            favoriteService.removeFromFavorites(media);
        }
    }

    @FXML
    private void onWatchedClicked(MouseEvent event) {
        event.consume();
        if (!media.isWatched()) {
            watchedService.addToWatchList(media);
        } else {
            watchedService.removeFromWatchList(media);
        }
    }

    @FXML
    private void close(MouseEvent event) {
        event.consume();
        eventPublisher.publishEvent(new CloseDetailsEvent(this));
        reset();
    }

    //endregion
}

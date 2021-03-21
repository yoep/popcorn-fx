package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerException;
import com.github.yoep.player.adapter.PlayerService;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.ui.media.favorites.FavoriteService;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.messages.SubtitleMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.controls.LanguageFlagCell;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controls.PlayerMenuItem;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.beans.value.ChangeListener;
import javafx.fxml.FXML;
import javafx.scene.control.*;
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
import java.util.stream.Collectors;

@Slf4j
public class MovieDetailsComponent extends AbstractDesktopDetailsComponent<Movie> {
    private static final String DEFAULT_TORRENT_AUDIO = "en";
    private static final String WATCHED_STYLE_CLASS = "seen";
    private static final String ACTIVE_PLAYER_STYLE_CLASS = "active";

    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final PlayerService playerService;

    private final ChangeListener<Boolean> watchedListener = (observable, oldValue, newValue) -> switchWatched(newValue);
    private final ChangeListener<Boolean> likedListener = (observable, oldValue, newValue) -> switchLiked(newValue);

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
    private Icon watchedIcon;
    @FXML
    private Tooltip watchedTooltip;
    @FXML
    private Tooltip favoriteTooltip;
    @FXML
    private SplitMenuButton watchNowButton;
    @FXML
    private Button watchTrailerButton;

    //region Constructors

    public MovieDetailsComponent(ApplicationEventPublisher eventPublisher, LocaleText localeText, HealthService healthService,
                                 SubtitleService subtitleService, SubtitlePickerService subtitlePickerService, ImageService imageService,
                                 SettingsService settingsService, FavoriteService favoriteService, WatchedService watchedService,
                                 PlayerService playerService) {
        super(eventPublisher, localeText, healthService, subtitleService, subtitlePickerService, imageService, settingsService);
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.playerService = playerService;
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
        if (media != null) {
            media.watchedProperty().removeListener(watchedListener);
            media.likedProperty().removeListener(likedListener);
        }

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

    private void initializeWatchNow() {
        // create initial list for the current known external players
        updateExternalPlayers();
        onActivePlayerChanged(playerService.getActivePlayer().orElse(null));

        // listen for changes in the players
        playerService.playersProperty().addListener((observable, oldValue, newValue) -> updateExternalPlayers());
        playerService.activePlayerProperty().addListener((observable, oldValue, newValue) -> onActivePlayerChanged(newValue));
    }

    private void updateExternalPlayers() {
        var items = playerService.getPlayers().stream()
                .map(this::playerToMenuItem)
                .collect(Collectors.toList());

        Platform.runLater(() -> watchNowButton.getItems().setAll(items));
    }

    private MenuItem playerToMenuItem(Player player) {
        var item = new PlayerMenuItem(player);
        item.setOnAction(e -> updateSelectedPlayer(item));
        return item;
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
    }

    private void loadFavoriteAndWatched() {
        switchLiked(favoriteService.isLiked(media));
        switchWatched(watchedService.isWatched(media));

        media.watchedProperty().addListener(watchedListener);
        media.likedProperty().addListener(likedListener);
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
    protected void switchLiked(boolean isLiked) {
        super.switchLiked(isLiked);

        if (isLiked) {
            favoriteTooltip.setText(localeText.get(DetailsMessage.REMOVE_FROM_BOOKMARKS));
        } else {
            favoriteTooltip.setText(localeText.get(DetailsMessage.ADD_TO_BOOKMARKS));
        }
    }

    private void switchWatched(boolean isWatched) {
        if (isWatched) {
            watchedIcon.setText(Icon.CHECK_UNICODE);
            watchedIcon.getStyleClass().add(WATCHED_STYLE_CLASS);
            watchedTooltip.setText(localeText.get(DetailsMessage.MARK_AS_NOT_SEEN));
        } else {
            watchedIcon.setText(Icon.EYE_SLASH_UNICODE);
            watchedIcon.getStyleClass().remove(WATCHED_STYLE_CLASS);
            watchedTooltip.setText(localeText.get(DetailsMessage.MARK_AS_SEEN));
        }
    }

    private void updateSelectedPlayer(MenuItem item) {
        var playerMenuItem = (PlayerMenuItem) item;
        var playerId = playerMenuItem.getId();
        var player = playerService.getById(playerId)
                .orElseThrow(() -> new PlayerException("Player not found with ID: " + playerId));

        // activate the player for usage
        playerService.setActivePlayer(player);
    }

    private void updateActivePlayerMenuItem(PlayerMenuItem item) {
        watchNowButton.getItems().forEach(e -> e.getStyleClass().removeIf(style -> style.equals(ACTIVE_PLAYER_STYLE_CLASS)));
        item.getStyleClass().add(ACTIVE_PLAYER_STYLE_CLASS);

        Optional.ofNullable(item.getImage())
                .map(ImageView::new)
                .ifPresent(watchNowButton::setGraphic);
    }

    private void onActivePlayerChanged(Player player) {
        if (player == null) {
            return;
        }

        var items = watchNowButton.getItems();

        items.stream()
                .filter(e -> e.getId().equals(player.getId()))
                .findFirst()
                .map(e -> (PlayerMenuItem) e)
                .ifPresentOrElse(
                        this::updateActivePlayerMenuItem,
                        () -> log.warn("Could not find menu item for player with ID {}", player.getId())
                );
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
        var mediaTorrentInfo = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);

        event.consume();
        eventPublisher.publishEvent(new LoadMediaTorrentEvent(this, mediaTorrentInfo, media, quality, subtitle));
    }

    @FXML
    private void onTrailerClicked(MouseEvent event) {
        event.consume();

        eventPublisher.publishEvent(new PlayVideoEvent(this, media.getTrailer(), media.getTitle(), false));
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

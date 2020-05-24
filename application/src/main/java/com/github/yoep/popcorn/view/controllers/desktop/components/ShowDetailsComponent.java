package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CloseDetailsActivity;
import com.github.yoep.popcorn.activities.LoadMediaTorrentActivity;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.media.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.models.Episode;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.media.watched.WatchedService;
import com.github.yoep.popcorn.media.watched.controls.WatchedCell;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.subtitles.controls.LanguageFlagCell;
import com.github.yoep.popcorn.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.view.controls.Episodes;
import com.github.yoep.popcorn.view.controls.Seasons;
import com.github.yoep.popcorn.view.models.Season;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.GridPane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.util.CollectionUtils;

import javax.annotation.PostConstruct;
import java.io.IOException;
import java.net.URL;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

@Slf4j
public class ShowDetailsComponent extends AbstractDesktopDetailsComponent<Show> {
    private static final DateTimeFormatter AIRED_DATE_PATTERN = DateTimeFormatter.ofPattern("EEEE, MMMM dd, yyyy hh:mm a");
    private static final double POSTER_WIDTH = 198.0;
    private static final double POSTER_HEIGHT = 215.0;

    private final FavoriteService favoriteService;
    private final WatchedService watchedService;

    private Episode episode;
    private boolean batchUpdating;

    @FXML
    private Label title;
    @FXML
    private Label year;
    @FXML
    private Label duration;
    @FXML
    private Label status;
    @FXML
    private Label genres;
    @FXML
    private Label overview;
    @FXML
    private Icon bookmarkIcon;
    @FXML
    private Label bookmark;
    @FXML
    private Seasons seasons;
    @FXML
    private Episodes episodes;
    @FXML
    private Label episodeTitle;
    @FXML
    private Label episodeSeason;
    @FXML
    private Label airDate;
    @FXML
    private Label episodeOverview;
    @FXML
    private GridPane episodeDetails;

    //region Constructors

    public ShowDetailsComponent(ActivityManager activityManager,
                                LocaleText localeText,
                                TorrentService torrentService,
                                SubtitleService subtitleService,
                                FavoriteService favoriteService,
                                WatchedService watchedService,
                                ImageService imageService,
                                SettingsService settingsService) {
        super(activityManager, localeText, torrentService, subtitleService, imageService, settingsService);
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void load(Show media) {
        super.load(media);

        loadText();
        loadSeasons();
        loadFavorite();
    }

    @Override
    protected CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return imageService.loadPoster(media, POSTER_WIDTH, POSTER_HEIGHT);
    }

    @Override
    protected void reset() {
        super.reset();
        resetLanguageSelection();

        title.setText(StringUtils.EMPTY);
        overview.setText(StringUtils.EMPTY);
        year.setText(StringUtils.EMPTY);
        duration.setText(StringUtils.EMPTY);
        status.setText(StringUtils.EMPTY);
        genres.setText(StringUtils.EMPTY);
        seasons.getItems().clear();
        episodes.getItems().clear();
        poster.setImage(null);
    }

    //endregion

    //region Initiazable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeSeasons();
        initializeEpisodes();
        initializeLanguageSelection();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    public void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(ShowSerieDetailsActivity.class, activity -> Platform.runLater(() -> load(activity.getMedia())));
    }

    //endregion

    //region Functions

    @Override
    protected void switchActiveQuality(String quality) {
        super.switchActiveQuality(quality);
        switchHealth(episode.getTorrents().get(quality));
    }

    private void initializeSeasons() {
        seasons.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSeason(newValue));
        seasons.setWatchedFactory(() -> new WatchedCell<>() {
            @Override
            protected void onItemChanged(Season oldItem, Season newItem) {
                super.onItemChanged(oldItem, newItem);

                if (newItem != null) {
                    if (!isSeasonEmpty(newItem)) {
                        boolean watched = isSeasonWatched(getWatchableItem());

                        setWatched(watched);
                        updateIcon(watched);
                        Tooltip.install(getIcon(), instantTooltip(getWatchedTooltip(watched)));

                        registerWatchedListener((observable, oldValue, newValue) -> onSeasonWatchedChanged(newValue, getWatchableItem(), getIcon()), newItem);
                    } else {
                        setGraphic(null);
                    }
                }
            }
        });
    }

    private void initializeEpisodes() {
        episodes.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchEpisode(newValue));
        episodes.setWatchedFactory(() -> new WatchedCell<>() {
            @Override
            protected void onItemChanged(Episode oldItem, Episode newItem) {
                super.onItemChanged(oldItem, newItem);

                if (newItem != null) {
                    boolean watched = watchedService.isWatched(getWatchableItem());

                    setWatched(watched);
                    updateIcon(watched);
                    Tooltip.install(getIcon(), instantTooltip(getWatchedTooltip(watched)));

                    registerWatchedListener((observable, oldValue, newValue) -> onEpisodeWatchedChanged(newValue, getWatchableItem(), getIcon()), newItem);
                }
            }
        });
    }

    private void initializeLanguageSelection() {
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

        languageSelection.addListener(newValue -> this.subtitle = newValue);
        resetLanguageSelection();
    }

    private void loadText() {
        title.setText(media.getTitle());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        status.setText(media.getStatus());
        genres.setText(String.join(" / ", media.getGenres()));
        overview.setText(media.getSynopsis());
    }

    private void loadSeasons() {
        for (int i = 1; i <= media.getNumberOfSeasons(); i++) {
            seasons.getItems().add(new Season(i, localeText.get(DetailsMessage.SEASON, i)));
        }

        selectUnwatchedSeason();
    }

    private void loadFavorite() {
        switchLiked(favoriteService.isLiked(media));
    }

    private void switchSeason(Season newSeason) {
        if (newSeason == null)
            return;

        List<Episode> episodes = getSeasonEpisodes(newSeason);

        this.episodes.getItems().clear();

        if (episodes.size() > 0) {
            this.episodeDetails.getChildren().forEach(e -> e.setVisible(true));
            this.episodes.getItems().addAll(episodes);
            selectUnwatchedEpisode();
        } else {
            this.episodeDetails.getChildren().forEach(e -> e.setVisible(false));
        }
    }

    private void switchEpisode(Episode episode) {
        if (episode == null)
            return;

        this.episode = episode;

        LocalDateTime airDateTime = episode.getAirDate();

        episodeTitle.setText(episode.getTitle());
        episodeSeason.setText(localeText.get(DetailsMessage.EPISODE_SEASON, episode.getSeason(), episode.getEpisode()));
        airDate.setText(localeText.get(DetailsMessage.AIR_DATE, AIRED_DATE_PATTERN.format(airDateTime)));
        episodeOverview.setText(episode.getSynopsis());

        loadQualitySelection(episode.getTorrents());
        loadSubtitles(episode);
    }

    private void loadSubtitles(Episode episode) {
        resetLanguageSelection();
        subtitleService.retrieveSubtitles(media, episode).whenComplete(this::handleSubtitlesResponse);
    }

    private List<Episode> getSeasonEpisodes(Season season) {
        return media.getEpisodes().stream()
                .filter(Objects::nonNull)
                .filter(e -> e.getSeason() == season.getSeason())
                .sorted(Comparator.comparing(Episode::getEpisode))
                .collect(Collectors.toList());
    }

    private boolean isSeasonEmpty(Season season) {
        return getSeasonEpisodes(season).size() == 0;
    }

    private boolean isSeasonWatched(Season season) {
        return getSeasonEpisodes(season).stream()
                .allMatch(watchedService::isWatched);
    }

    private void markSeasonAsWatched(Season season) {
        batchUpdating = true;
        getSeasonEpisodes(season).forEach(e -> e.setWatched(true));
        batchUpdating = false;
    }

    private void unmarkSeasonAsWatched(Season season) {
        batchUpdating = true;
        getSeasonEpisodes(season).forEach(e -> e.setWatched(false));
        batchUpdating = false;
    }

    private void selectUnwatchedSeason() {
        var seasons = this.seasons.getItems();
        var season = seasons.stream()
                .filter(e -> !isSeasonWatched(e))
                .findFirst()
                .orElseGet(() -> CollectionUtils.lastElement(seasons));

        Platform.runLater(() -> {
            this.seasons.getSelectionModel().select(season);
            this.seasons.scrollTo(season);
        });
    }

    private void selectUnwatchedEpisode() {
        var episodes = this.episodes.getItems();
        var episode = episodes.stream()
                .filter(Objects::nonNull)
                .filter(e -> !watchedService.isWatched(e))
                .findFirst()
                .orElseGet(() -> {
                    // check if the current season should be marked as watched
                    updateSeasonIfNeeded(this.seasons.getSelectionModel().getSelectedItem());

                    return CollectionUtils.lastElement(episodes);
                });

        Platform.runLater(() -> {
            this.episodes.getSelectionModel().select(episode);
            this.episodes.scrollTo(episode);
        });
    }

    private String getWatchedTooltip(boolean watched) {
        return localeText.get(watched ? DetailsMessage.UNMARK_AS_WATCHED : DetailsMessage.MARK_AS_WATCHED);
    }

    private void onSeasonWatchedChanged(Boolean newValue, Season season, Icon icon) {
        Tooltip.install(icon, instantTooltip(getWatchedTooltip(newValue)));

        if (newValue) {
            markSeasonAsWatched(season);
        } else {
            unmarkSeasonAsWatched(season);
        }

        // navigate to the next unwatched season
        selectUnwatchedSeason();
    }

    private void onEpisodeWatchedChanged(Boolean newValue, Episode episode, Icon icon) {
        Tooltip.install(icon, instantTooltip(getWatchedTooltip(newValue)));

        if (newValue) {
            watchedService.addToWatchList(episode);
        } else {
            watchedService.removeFromWatchList(episode);
        }

        // check if a batch update is running
        // if so, do not go to the next unwatched episode
        if (!batchUpdating) {
            // navigate to the next unwatched episode
            selectUnwatchedEpisode();
        }
    }

    private void updateSeasonIfNeeded(Season season) {
        if (isSeasonWatched(season))
            season.setWatched(true);
    }

    @FXML
    private void onMagnetClicked(MouseEvent event) {
        MediaTorrentInfo torrentInfo = episode.getTorrents().get(quality);

        if (event.getButton() == MouseButton.SECONDARY) {
            copyMagnetLink(torrentInfo);
        } else {
            openMagnetLink(torrentInfo);
        }
    }

    @FXML
    private void onWatchNowClicked() {
        activityManager.register(new LoadMediaTorrentActivity() {
            @Override
            public MediaTorrentInfo getTorrent() {
                return episode.getTorrents().get(quality);
            }

            @Override
            public Media getMedia() {
                return episode;
            }

            @Override
            public String getQuality() {
                return quality;
            }

            @Override
            public Optional<SubtitleInfo> getSubtitle() {
                return Optional.ofNullable(subtitle);
            }
        });
    }

    @FXML
    private void close() {
        activityManager.register(new CloseDetailsActivity() {
        });
        reset();
    }

    //endregion
}

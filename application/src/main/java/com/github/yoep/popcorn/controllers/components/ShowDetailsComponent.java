package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CloseDetailsActivity;
import com.github.yoep.popcorn.activities.LoadTorrentActivity;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.controls.Episodes;
import com.github.yoep.popcorn.controls.Seasons;
import com.github.yoep.popcorn.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.models.Episode;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.media.providers.models.TorrentInfo;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.models.Season;
import com.github.yoep.popcorn.subtitle.SubtitleService;
import com.github.yoep.popcorn.subtitle.controls.LanguageFlagCell;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.watched.WatchedService;
import com.github.yoep.popcorn.watched.controls.WatchedCell;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.collections.ObservableList;
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
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;
import org.springframework.util.CollectionUtils;

import javax.annotation.PostConstruct;
import java.io.IOException;
import java.net.URL;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.util.*;
import java.util.stream.Collectors;

@Slf4j
@Component
public class ShowDetailsComponent extends AbstractDetailsComponent<Show> {
    private static final DateTimeFormatter AIRED_DATE_PATTERN = DateTimeFormatter.ofPattern("EEEE, MMMM dd, yyyy hh:mm a");

    private final ActivityManager activityManager;
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final SubtitleService subtitleService;

    private Episode episode;

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
                                TaskExecutor taskExecutor,
                                LocaleText localeText,
                                TorrentService torrentService,
                                Application application,
                                FavoriteService favoriteService,
                                WatchedService watchedService, SubtitleService subtitleService) {
        super(taskExecutor, localeText, torrentService, application);
        this.activityManager = activityManager;
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.subtitleService = subtitleService;
    }

    //endregion

    //region AbstractDetailsComponent

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

    //region Methods

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePoster();
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

    private void load(Show media) {
        Assert.notNull(media, "media cannot be null");
        reset();

        this.media = media;

        loadText();
        loadStars();
        loadSeasons();
        loadFavorite();
        loadPosterImage();
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
        switchFavorite(favoriteService.isFavorite(media));
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

        LocalDateTime airDateTime = LocalDateTime.ofInstant(Instant.ofEpochSecond(episode.getFirstAired()), ZoneOffset.UTC);

        episodeTitle.setText(episode.getTitle());
        episodeSeason.setText(localeText.get(DetailsMessage.EPISODE_SEASON, episode.getSeason(), episode.getEpisode()));
        airDate.setText(localeText.get(DetailsMessage.AIR_DATE, AIRED_DATE_PATTERN.format(airDateTime)));
        episodeOverview.setText(episode.getOverview());

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
        getSeasonEpisodes(season).forEach(e -> e.setWatched(true));
    }

    private void unmarkSeasonAsWatched(Season season) {
        getSeasonEpisodes(season).forEach(e -> e.setWatched(false));
    }

    private void selectUnwatchedSeason() {
        ObservableList<Season> seasons = this.seasons.getItems();
        Season season = seasons.stream()
                .filter(e -> !isSeasonWatched(e))
                .findFirst()
                .orElseGet(() -> CollectionUtils.lastElement(seasons));

        Platform.runLater(() -> this.seasons.getSelectionModel().select(season));
    }

    private void selectUnwatchedEpisode() {
        ObservableList<Episode> episodes = this.episodes.getItems();
        Episode episode = episodes.stream()
                .filter(Objects::nonNull)
                .filter(e -> !watchedService.isWatched(e))
                .findFirst()
                .orElseGet(() -> CollectionUtils.lastElement(episodes));

        Platform.runLater(() -> this.episodes.getSelectionModel().select(episode));
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

        // navigate to the next unwatched episode
        selectUnwatchedEpisode();
    }

    @FXML
    private void onMagnetClicked(MouseEvent event) {
        TorrentInfo torrentInfo = episode.getTorrents().get(quality);

        if (event.getButton() == MouseButton.SECONDARY) {
            copyMagnetLink(torrentInfo);
        } else {
            openMagnetLink(torrentInfo);
        }
    }

    @FXML
    private void onWatchNowClicked() {
        activityManager.register(new LoadTorrentActivity() {
            @Override
            public String getQuality() {
                return quality;
            }

            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public TorrentInfo getTorrent() {
                return episode.getTorrents().get(quality);
            }

            @Override
            public Optional<Episode> getEpisode() {
                return Optional.of(episode);
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

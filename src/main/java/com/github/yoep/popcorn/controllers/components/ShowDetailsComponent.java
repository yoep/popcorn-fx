package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.CloseDetailsActivity;
import com.github.yoep.popcorn.activities.LoadTorrentActivity;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.controls.Episodes;
import com.github.yoep.popcorn.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.models.Episode;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.media.providers.models.TorrentInfo;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.models.Season;
import com.github.yoep.popcorn.subtitle.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.watched.WatchedService;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.util.Comparator;
import java.util.Objects;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.stream.Collectors;

@Slf4j
@Component
public class ShowDetailsComponent extends AbstractDetailsComponent<Show> {
    private static final DateTimeFormatter AIRED_DATE_PATTERN = DateTimeFormatter.ofPattern("EEEE, MMMM dd, yyyy hh:mm a");

    private final ActivityManager activityManager;
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;

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
    private ListView<Season> seasons;
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
    private LanguageFlagSelection languageSelection;

    //region Constructors

    public ShowDetailsComponent(ActivityManager activityManager,
                                TaskExecutor taskExecutor,
                                LocaleText localeText,
                                TorrentService torrentService,
                                Application application,
                                FavoriteService favoriteService,
                                WatchedService watchedService) {
        super(taskExecutor, localeText, torrentService, application);
        this.activityManager = activityManager;
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void reset() {
        super.reset();

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

    //region Functions

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePoster();
        initializeListViews();
    }

    @PostConstruct
    public void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(ShowSerieDetailsActivity.class, activity -> Platform.runLater(() -> load(activity.getMedia())));
    }

    private void initializeListViews() {
        seasons.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSeason(newValue));
        episodes.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchEpisode(newValue));
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

        seasons.getSelectionModel().select(0);
    }

    private void loadFavorite() {
        switchFavorite(favoriteService.isFavorite(media));
    }

    @Override
    protected void switchActiveQuality(String quality) {
        super.switchActiveQuality(quality);
        switchHealth(episode.getTorrents().get(quality));
    }

    private void switchSeason(Season newSeason) {
        if (newSeason == null)
            return;

        episodes.getItems().clear();
        episodes.getItems().addAll(media.getEpisodes().stream()
                .filter(Objects::nonNull)
                .filter(e -> e.getSeason() == newSeason.getSeason())
                .sorted(Comparator.comparing(Episode::getEpisode))
                .collect(Collectors.toList()));
        episodes.getSelectionModel().select(0);
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
            public Optional<SubtitleInfo> getSubtitle() {
                return Optional.ofNullable(subtitle);
            }
        });
    }

    @FXML
    private void onSubtitleLabelClicked() {

    }

    @FXML
    private void close() {
        activityManager.register(new CloseDetailsActivity() {
        });
        reset();
    }

    //endregion
}

package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.filters.model.Season;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.controls.LanguageFlagCell;
import com.github.yoep.popcorn.ui.controls.WatchedCell;
import com.github.yoep.popcorn.ui.controls.WatchedCellCallbacks;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.controls.Episodes;
import com.github.yoep.popcorn.ui.view.controls.Seasons;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.ShowHelperService;
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

import java.io.IOException;
import java.net.URL;
import java.time.LocalDateTime;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
@ViewController
public class ShowDetailsComponent extends AbstractDesktopDetailsComponent<ShowDetails> {
    private final ShowHelperService showHelperService;

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

    public ShowDetailsComponent(EventPublisher eventPublisher,
                                LocaleText localeText,
                                HealthService healthService,
                                SubtitleService subtitleService,
                                SubtitlePickerService subtitlePickerService,
                                ImageService imageService,
                                ApplicationConfig settingsService,
                                DetailsComponentService service,
                                ShowHelperService showHelperService,
                                FxLib fxLib) {
        super(eventPublisher,
                localeText,
                healthService,
                subtitleService,
                subtitlePickerService,
                imageService,
                settingsService,
                service,
                fxLib);

        this.showHelperService = showHelperService;
        service.addListener(createCallback());
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void load(ShowDetails media) {
        super.load(media);

        loadText();
        loadSeasons();
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
    }

    //endregion

    //region Initiazable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializeSeasons();
        initializeEpisodes();
        initializeLanguageSelection();
        initializeTooltips();
        initializeListeners();
    }

    private void initializeListeners() {
        eventPublisher.register(ShowSerieDetailsEvent.class, event -> {
            Platform.runLater(() -> load(event.getMedia()));
            return event;
        });
    }

    //endregion

    //region Functions

    @Override
    protected void switchActiveQuality(String quality) {
        super.switchActiveQuality(quality);
        switchHealth(episode.getTorrents().get(quality));
    }

    private void initializeSeasons() {
        var seasonsCallback = new WatchedCellCallbacks() {
            @Override
            public void updateWatchedState(Media media, boolean newState, Icon icon) {
                onSeasonWatchedChanged(newState, (Season) media, icon);
            }
        };

        seasons.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSeason(newValue));
        seasons.setWatchedFactory(() -> new WatchedCell<>(seasonsCallback) {
            @Override
            protected void onItemChanged(Season newItem) {
                super.onItemChanged(newItem);

                if (newItem != null) {
                    if (!isSeasonEmpty(newItem)) {
                        boolean watched = isSeasonWatched(getWatchableItem());

                        updateIcon(watched);
                        Tooltip.install(getIcon(), instantTooltip(getWatchedTooltip(watched)));
                    } else {
                        setGraphic(null);
                    }
                }
            }
        });
    }

    private void initializeEpisodes() {
        var cellCallbacks = new WatchedCellCallbacks() {
            @Override
            public void updateWatchedState(Media media, boolean newState, Icon icon) {
                service.updateWatchedStated(media, newState);
                onEpisodeWatchedChanged(newState, (Episode) media, icon);
            }
        };

        episodes.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchEpisode(newValue));
        episodes.setWatchedFactory(() -> new WatchedCell<>(cellCallbacks) {
            @Override
            protected void onItemChanged(Episode newItem) {
                super.onItemChanged(newItem);

                if (newItem != null) {
                    boolean watched = service.isWatched(getWatchableItem());

                    updateIcon(watched);
                    Tooltip.install(getIcon(), instantTooltip(getWatchedTooltip(watched)));
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

        languageSelection.addListener(createLanguageListener());
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
        seasons.getItems().addAll(showHelperService.getSeasons(media));
        selectUnwatchedSeason();
    }

    private void switchSeason(Season newSeason) {
        if (newSeason == null)
            return;

        List<Episode> episodes = showHelperService.getSeasonEpisodes(newSeason, media);

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
        this.episode = episode;

        if (episode == null)
            return;

        log.trace("Show episode has been switched to {}", episode);
        LocalDateTime airDateTime = episode.getAirDate();

        episodeTitle.setText(episode.getTitle());
        episodeSeason.setText(localeText.get(DetailsMessage.EPISODE_SEASON, episode.getSeason(), episode.getEpisode()));
        airDate.setText(localeText.get(DetailsMessage.AIR_DATE, ShowHelperService.AIRED_DATE_PATTERN.format(airDateTime)));
        episodeOverview.setText(episode.getSynopsis());

        loadQualitySelection(episode.getTorrents());
        loadSubtitles(episode);
    }

    private void loadSubtitles(Episode episode) {
        resetLanguageSelection();
        subtitleService.retrieveSubtitles(media, episode).whenComplete(this::handleSubtitlesResponse);
    }

    private boolean isSeasonEmpty(Season season) {
        return showHelperService.getSeasonEpisodes(season, media).size() == 0;
    }

    private boolean isSeasonWatched(Season season) {
        return showHelperService.getSeasonEpisodes(season, media).stream()
                .allMatch(service::isWatched);
    }

    private void markSeasonAsWatched(Season season) {
        batchUpdating = true;
        showHelperService.getSeasonEpisodes(season, media).forEach(e -> service.updateWatchedStated(e, true));
        batchUpdating = false;
    }

    private void unmarkSeasonAsWatched(Season season) {
        batchUpdating = true;
        showHelperService.getSeasonEpisodes(season, media).forEach(e -> service.updateWatchedStated(e, false));
        batchUpdating = false;
    }

    private void selectUnwatchedSeason() {
        var seasons = this.seasons.getItems();
        var season = showHelperService.getUnwatchedSeason(seasons, media);

        Platform.runLater(() -> {
            this.seasons.getSelectionModel().select(season);
            this.seasons.scrollTo(season);
        });
    }

    private void selectUnwatchedEpisode() {
        var episodes = this.episodes.getItems();
        var episode = showHelperService.getUnwatchedEpisode(episodes);

        // check if the current season should be marked as watched
        updateSeasonIfNeeded(this.seasons.getSelectionModel().getSelectedItem());

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
        seasons.updateWatchedState(season, newValue);
        selectUnwatchedSeason();
    }

    private void onEpisodeWatchedChanged(Boolean newValue, Episode episode, Icon icon) {
        Tooltip.install(icon, instantTooltip(getWatchedTooltip(newValue)));
        service.updateWatchedStated(episode, newValue);

        // check if a batch update is running
        // if so, do not go to the next unwatched episode
        if (!batchUpdating) {
            // navigate to the next unwatched episode
            selectUnwatchedEpisode();
        }
    }

    private void updateSeasonIfNeeded(Season season) {
        seasons.updateWatchedState(season, isSeasonWatched(season));
    }

    private DetailsComponentListener createCallback() {
        return new DetailsComponentListener() {
            @Override
            public void onWatchChanged(String id, boolean newState) {
                episodes.getItems().stream()
                        .filter(e -> e.getId().equals(id))
                        .findFirst()
                        .ifPresent(e -> episodes.updateWatchedState(e, newState));
            }

            @Override
            public void onLikedChanged(boolean newState) {
                // no-op
            }
        };
    }

    @FXML
    void onMagnetClicked(MouseEvent event) {
        MediaTorrentInfo torrentInfo = episode.getTorrents().get(quality);

        if (event.getButton() == MouseButton.SECONDARY) {
            copyMagnetLink(torrentInfo);
        } else {
            openMagnetLink(torrentInfo);
        }
    }

    @FXML
    void onWatchedClicked(MouseEvent event) {
        event.consume();
        service.toggleWatchedState();
    }

    @FXML
    void onFavoriteClicked(MouseEvent event) {
        event.consume();
        service.toggleLikedState();
    }

    @FXML
    void onWatchNowClicked() {
        var mediaTorrentInfo = episode.getTorrents().get(quality);

        eventPublisher.publishEvent(new LoadMediaTorrentEvent(this, mediaTorrentInfo, media, episode, quality, subtitle));
    }

    //endregion
}

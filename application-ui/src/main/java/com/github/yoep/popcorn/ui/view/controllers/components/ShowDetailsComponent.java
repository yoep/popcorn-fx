package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
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
import com.github.yoep.popcorn.ui.controls.WatchedCellCallbacks;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.ShowHelperService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
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
    private final ViewLoader viewLoader;

    private Episode episode;
    private boolean batchUpdating;

    @FXML
    GridPane showDetails;
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
    AxisItemSelection<Season> seasons;
    @FXML
    AxisItemSelection<Episode> episodes;
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
                                FxLib fxLib,
                                ViewLoader viewLoader) {
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
        this.viewLoader = viewLoader;
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
        seasons.setItems();
        seasons.setItems();
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
        initializePoster();
        initializeMode();
    }

    private void initializeListeners() {
        eventPublisher.register(ShowSerieDetailsEvent.class, event -> {
            Platform.runLater(() -> load(event.getMedia()));
            return event;
        });
    }

    private void initializePoster() {
        var poster = viewLoader.load("components/poster.component.fxml");
        showDetails.add(poster, 0, 0, 1, 3);
    }

    private void initializeMode() {
        showDetails.getColumnConstraints().get(0).setMinWidth(service.isTvMode() ? 285.0 : 190.0);
        AnchorPane.setLeftAnchor(showDetails, service.isTvMode() ? 150.0 : 75.0);
    }

    //endregion

    //region Functions

    @Override
    protected void switchActiveQuality(String quality) {
        super.switchActiveQuality(quality);
        switchHealth(episode.getTorrents().get(quality));
    }

    private void initializeSeasons() {
        seasons.selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSeason(newValue));
        seasons.setFactory(item -> {
            var styleClass = isSeasonWatched(item) ? "watched" : null;
            var icon = new Icon(Icon.EYE_UNICODE);

            icon.setOnMouseClicked(event -> {
                event.consume();
                onSeasonWatchedChanged(!isSeasonWatched(item), item, icon);
            });
            icon.getStyleClass().add(styleClass);
            return new Button(item.getText(), icon);
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

        episodes.selectedItemProperty().addListener((observable, oldValue, newValue) -> switchEpisode(newValue));
        episodes.setFactory(item -> viewLoader.load("common/components/episode.component.fxml", new EpisodeComponent(item, localeText, imageService)));
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
        seasons.setItems(showHelperService.getSeasons(media).toArray(new Season[0]));
        selectUnwatchedSeason();
    }

    private void switchSeason(Season newSeason) {
        if (newSeason == null)
            return;

        List<Episode> episodes = showHelperService.getSeasonEpisodes(newSeason, media);

        if (episodes.size() > 0) {
            this.episodeDetails.getChildren().forEach(e -> e.setVisible(true));
            this.episodes.setItems(episodes.toArray(new Episode[0]));
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

        Platform.runLater(() -> this.seasons.setSelectedItem(season));
    }

    private void selectUnwatchedEpisode() {
        var episodes = this.episodes.getItems();
        var episode = showHelperService.getUnwatchedEpisode(episodes);

        // check if the current season should be marked as watched
        updateSeasonIfNeeded(this.seasons.getSelectedItem());
        Platform.runLater(() -> {
            this.episodes.setSelectedItem(episode);
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

        seasons.setItems(showHelperService.getSeasons(media).toArray(new Season[0]));
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
        // todo: update
        //        seasons.updateWatchedState(season, isSeasonWatched(season));
    }

    private DetailsComponentListener createCallback() {
        return new DetailsComponentListener() {
            @Override
            public void onWatchChanged(String id, boolean newState) {
                //todo: update episode state
                //                episodes.getItems().stream()
                //                        .filter(e -> e.getId().equals(id))
                //                        .findFirst()
                //                        .ifPresent(e -> episodes.updateWatchedState(e, newState));
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

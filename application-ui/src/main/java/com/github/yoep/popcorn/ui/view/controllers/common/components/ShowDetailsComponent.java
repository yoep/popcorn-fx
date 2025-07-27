package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.media.Season;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.MediaQualityChangedEvent;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.ViewHelper;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.*;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.GridPane;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static java.util.Arrays.asList;

@Slf4j

public class ShowDetailsComponent extends AbstractDesktopDetailsComponent<ShowDetails> {
    static final String POSTER_COMPONENT_FXML = "components/poster.component.fxml";
    static final String SERIE_ACTIONS_COMPONENT_FXML = "components/serie-actions.component.fxml";
    static final String EPISODE_COMPONENT_FXML = "common/components/episode.component.fxml";
    static final String EPISODE_ACTIONS_COMPONENT_FXML = "components/serie-episode-actions.component.fxml";

    private final ShowHelperService showHelperService;
    private final ViewLoader viewLoader;
    private final SerieActionsComponent serieActionsComponent;
    private final VideoQualityService videoQualityService;

    Episode episode;
    String quality;

    @FXML
    GridPane showDetails;
    @FXML
    Label title;
    @FXML
    Label year;
    @FXML
    Label duration;
    @FXML
    Label status;
    @FXML
    Label genres;
    @FXML
    Label overview;
    @FXML
    AxisItemSelection<Season> seasons;
    @FXML
    AxisItemSelection<Episode> episodes;
    @FXML
    Overlay episodeDetailsOverlay;
    @FXML
    GridPane episodeDetails;
    @FXML
    Label episodeTitle;
    @FXML
    Label episodeSeason;
    @FXML
    Label airDate;
    @FXML
    Label synopsis;

    //region Constructors

    public ShowDetailsComponent(EventPublisher eventPublisher,
                                LocaleText localeText,
                                HealthService healthService,
                                ISubtitleService subtitleService,
                                SubtitlePickerService subtitlePickerService,
                                ImageService imageService,
                                ApplicationConfig settingsService,
                                DetailsComponentService service,
                                ShowHelperService showHelperService,
                                ViewLoader viewLoader,
                                SerieActionsComponent serieActionsComponent,
                                VideoQualityService videoQualityService) {
        super(eventPublisher,
                localeText,
                healthService,
                subtitleService,
                subtitlePickerService,
                imageService,
                settingsService,
                service);

        this.showHelperService = showHelperService;
        this.viewLoader = viewLoader;
        this.serieActionsComponent = serieActionsComponent;
        this.videoQualityService = videoQualityService;
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void load(ShowDetails media) {
        super.load(media);

        loadText();
        loadSeasons();
        episodeDetailsOverlay.hide();
    }

    @Override
    protected void reset() {
        super.reset();
        title.setText(null);
        overview.setText(null);
        year.setText(null);
        duration.setText(null);
        status.setText(null);
        genres.setText(null);
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
        initializeTooltips();
        initializeListeners();
        initializePoster();
        initializeMode();
        initializeSerieActions();
    }

    private void initializeListeners() {
        eventPublisher.register(ShowSerieDetailsEvent.class, event -> {
            Platform.runLater(() -> load(event.getMedia()));
            return event;
        });
        eventPublisher.register(MediaQualityChangedEvent.class, event -> {
            Platform.runLater(() -> {
                if (episode != null && (event.getMedia() instanceof ShowDetails || event.getMedia() instanceof Episode)) {
                    switchHealth(episode.getTorrents().getQualitiesMap().get(event.getQuality()));
                }
            });
            this.quality = event.getQuality();
            return event;
        });
    }

    private void initializePoster() {
        var poster = viewLoader.load(POSTER_COMPONENT_FXML);
        showDetails.add(poster, 0, 0, 1, 4);
    }

    private void initializeMode() {
        showDetails.getColumnConstraints().get(0).setMinWidth(service.isTvMode() ? 285.0 : 190.0);
        AnchorPane.setLeftAnchor(showDetails, service.isTvMode() ? 150.0 : 75.0);
    }

    private void initializeSerieActions() {
        var actions = viewLoader.load(SERIE_ACTIONS_COMPONENT_FXML);
        showDetails.add(actions, 2, 3);
    }

    //endregion

    //region Functions

    private void initializeSeasons() {
        seasons.selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSeason(newValue));
        seasons.setItemFactory(item -> {
            var icon = new Icon(Icon.EYE_UNICODE);

            isSeasonWatched(item).whenComplete((watched, throwable) -> {
                if (throwable == null) {
                    var styleClass = watched ? "watched" : null;

                    icon.setOnMouseClicked(event -> {
                        event.consume();
                        onSeasonWatchedChanged(!watched, item, icon);
                    });
                    icon.getStyleClass().add(styleClass);
                } else {
                    log.error("Failed to retrieve is watched", throwable);
                }
            });

            return new Button(item.title(), icon);
        });
    }

    private void initializeEpisodes() {
        episodes.setOnItemActivated(this::switchEpisode);
        episodes.setItemFactory(item -> {
            var controller = new EpisodeComponent(item, localeText, imageService);
            var listener = episodeWatchStateListener(item, controller);

            service.isWatched(item).whenComplete((watched, throwable) -> {
                if (throwable == null) {
                    controller.updateWatchedState(watched);
                } else {
                    log.error("Failed to retrieve is watched", throwable);
                }
            });
            controller.setOnWatchClicked(newState -> service.updateWatchedStated(item, newState));
            controller.setOnDestroy(() -> service.removeListener(listener));
            service.addListener(listener);

            return viewLoader.load(EPISODE_COMPONENT_FXML, controller);
        });

        var episodeActions = viewLoader.load(EPISODE_ACTIONS_COMPONENT_FXML);
        episodeDetails.add(episodeActions, 0, 4, 2, 1);
        serieActionsComponent.setOnWatchNowClicked(() -> episodeDetailsOverlay.hide());
    }

    private void loadText() {
        title.setText(media.title());
        year.setText(media.year());
        duration.setText(media.runtime() + " min");
        status.setText(media.proto().getStatus());
        genres.setText(String.join(" / ", media.genres()));
        overview.setText(media.synopsis());
    }

    private void loadSeasons() {
        seasons.setItems(showHelperService.getSeasons(media).stream()
                .filter(e -> !showHelperService.getSeasonEpisodes(e, media).isEmpty())
                .toArray(Season[]::new));
        selectUnwatchedSeason();
    }

    private void switchSeason(Season newSeason) {
        if (newSeason == null)
            return;

        List<Episode> episodes = showHelperService.getSeasonEpisodes(newSeason, media);

        this.episodes.setItems(episodes.toArray(new Episode[0]));
        if (!episodes.isEmpty()) {
            selectUnwatchedEpisode(newSeason);
        }
    }

    private void switchEpisode(Episode episode) {
        this.episodeDetailsOverlay.show();
        this.episode = episode;

        if (episode == null)
            return;

        log.trace("Show episode has been switched to {}", episode);
        episodeTitle.setText(episode.title());
        episodeSeason.setText(localeText.get(DetailsMessage.EPISODE_SEASON, episode.season(), episode.episode()));
        airDate.setText(localeText.get(DetailsMessage.AIR_DATE, ShowHelperService.AIRED_DATE_PATTERN.format(episode.getAirDate())));
        synopsis.setText(episode.synopsis());

        serieActionsComponent.episodeChanged(media, episode);
    }

    private CompletableFuture<Boolean> isSeasonWatched(Season season) {
        List<CompletableFuture<Boolean>> futures = showHelperService.getSeasonEpisodes(season, media).stream()
                .map(service::isWatched)
                .toList();

        return CompletableFuture.allOf(futures.toArray(new CompletableFuture[0]))
                .thenApply(v -> futures.stream()
                        .allMatch(CompletableFuture::join));
    }

    private void markSeasonAsWatched(Season season) {
        showHelperService.getSeasonEpisodes(season, media).forEach(e -> service.updateWatchedStated(e, true));
    }

    private void unmarkSeasonAsWatched(Season season) {
        showHelperService.getSeasonEpisodes(season, media).forEach(e -> service.updateWatchedStated(e, false));
    }

    private void selectUnwatchedSeason() {
        var seasons = this.seasons.getItems();
        showHelperService.getUnwatchedSeason(seasons, media)
                .thenAccept(season -> {
                    Platform.runLater(() -> this.seasons.setSelectedItem(season));
                    selectUnwatchedEpisode(season);
                })
                .exceptionally(throwable -> {
                    log.error("Failed to load unwatched season", throwable);
                    return null;
                });
    }

    private void selectUnwatchedEpisode(Season newSeason) {
        var episodes = this.episodes.getItems();
        showHelperService.getUnwatchedEpisode(episodes, newSeason)
                .thenAccept(episode ->
                        episode.ifPresent(e -> Platform.runLater(() -> this.episodes.setSelectedItem(e, true)))
                ).exceptionally(ex -> {
                    log.error("Failed to get unwatched episode, {}", ex.getMessage(), ex);
                    return null;
                });
    }

    private String getWatchedTooltip(boolean watched) {
        return localeText.get(watched ? DetailsMessage.UNMARK_AS_WATCHED : DetailsMessage.MARK_AS_WATCHED);
    }

    private void onSeasonWatchedChanged(Boolean newValue, Season season, Icon icon) {
        Tooltip.install(icon, ViewHelper.instantTooltip(getWatchedTooltip(newValue)));

        if (newValue) {
            markSeasonAsWatched(season);
        } else {
            unmarkSeasonAsWatched(season);
        }

        seasons.setItems(showHelperService.getSeasons(media).toArray(new Season[0]));
        selectUnwatchedSeason();
    }

    private DetailsComponentListener episodeWatchStateListener(Episode item, EpisodeComponent controller) {
        return new DetailsComponentListener() {
            @Override
            public void onWatchChanged(String imdbId, boolean newState) {
                if (imdbId.equals(item.id())) {
                    controller.updateWatchedState(newState);
                } else {
                    selectUnwatchedSeason();
                }
            }

            @Override
            public void onLikedChanged(String imdbId, boolean newState) {
                // no-op
            }
        };
    }

    @FXML
    void onMagnetClicked(MouseEvent event) {
        event.consume();
        var qualities = videoQualityService.getVideoResolutions(episode.getTorrents());
        var qualityFuture = Optional.ofNullable(this.quality)
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> videoQualityService.getDefaultVideoResolution(asList(qualities)));

        qualityFuture.whenComplete((quality, throwable) -> {
            if (throwable == null) {
                var torrentInfo = episode.getTorrents().getQualitiesMap().get(quality);
                if (event.getButton() == MouseButton.SECONDARY) {
                    copyMagnetLink(torrentInfo);
                } else {
                    openMagnetLink(torrentInfo);
                }
            } else {
                log.error("Failed to get video resolution", throwable);
            }
        });
    }

    @FXML
    void onWatchedClicked(MouseEvent event) {
        event.consume();
        service.toggleWatchedState(media);
    }

    @FXML
    void onFavoriteClicked(MouseEvent event) {
        event.consume();
        service.toggleLikedState(media);
    }

    //endregion
}

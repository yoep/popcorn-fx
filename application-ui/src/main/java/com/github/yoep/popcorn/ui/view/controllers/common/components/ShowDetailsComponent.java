package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.filters.model.Season;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.events.MediaQualityChangedEvent;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.ViewHelper;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
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
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.GridPane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
@ViewController
public class ShowDetailsComponent extends AbstractDesktopDetailsComponent<ShowDetails> {
    private final ShowHelperService showHelperService;
    private final ViewLoader viewLoader;
    private final SerieActionsComponent serieActionsComponent;

    private Episode episode;

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
    Overlay overlay;
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
                                SubtitleService subtitleService,
                                SubtitlePickerService subtitlePickerService,
                                ImageService imageService,
                                ApplicationConfig settingsService,
                                DetailsComponentService service,
                                ShowHelperService showHelperService,
                                ViewLoader viewLoader,
                                SerieActionsComponent serieActionsComponent) {
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
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void load(ShowDetails media) {
        super.load(media);

        loadText();
        loadSeasons();
        overlay.hide();
    }

    @Override
    protected void reset() {
        super.reset();
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
                if (episode != null && event.getMedia() instanceof ShowDetails) {
                    switchHealth(episode.getTorrents().get(event.getQuality()));
                }
            });
            return event;
        });
    }

    private void initializePoster() {
        var poster = viewLoader.load("components/poster.component.fxml");
        showDetails.add(poster, 0, 0, 1, 4);
    }

    private void initializeMode() {
        showDetails.getColumnConstraints().get(0).setMinWidth(service.isTvMode() ? 285.0 : 190.0);
        AnchorPane.setLeftAnchor(showDetails, service.isTvMode() ? 150.0 : 75.0);
    }

    private void initializeSerieActions() {
        var actions = viewLoader.load("components/serie-actions.component.fxml");
        showDetails.add(actions, 2, 3);
    }

    //endregion

    //region Functions

    private void initializeSeasons() {
        seasons.selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSeason(newValue));
        seasons.setItemFactory(item -> {
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
        episodes.setOnItemActivated(this::switchEpisode);
        episodes.setItemFactory(item -> {
            var controller = new EpisodeComponent(item, localeText, imageService);
            var listener = episodeWatchStateListener(item, controller);

            controller.updateWatchedState(service.isWatched(item));
            controller.setOnWatchClicked(newState -> service.updateWatchedStated(item, newState));
            controller.setOnDestroy(() -> service.removeListener(listener));
            service.addListener(listener);

            return viewLoader.load("common/components/episode.component.fxml", controller);
        });

        var episodeActions = viewLoader.load("components/serie-episode-actions.component.fxml");
        episodeDetails.add(episodeActions, 0, 4, 2, 1);
        serieActionsComponent.setOnWatchNowClicked(() -> overlay.hide());
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
            this.episodes.setItems(episodes.toArray(new Episode[0]));
            selectUnwatchedEpisode(newSeason);
        }
    }

    private void switchEpisode(Episode episode) {
        this.overlay.show();
        this.episode = episode;

        if (episode == null)
            return;

        log.trace("Show episode has been switched to {}", episode);
        episodeTitle.setText(episode.getTitle());
        episodeSeason.setText(localeText.get(DetailsMessage.EPISODE_SEASON, episode.getSeason(), episode.getEpisode()));
        airDate.setText(localeText.get(DetailsMessage.AIR_DATE, ShowHelperService.AIRED_DATE_PATTERN.format(episode.getAirDate())));
        synopsis.setText(episode.getSynopsis());

        serieActionsComponent.episodeChanged(media, episode);
    }

    private boolean isSeasonWatched(Season season) {
        return showHelperService.getSeasonEpisodes(season, media).stream()
                .allMatch(service::isWatched);
    }

    private void markSeasonAsWatched(Season season) {
        showHelperService.getSeasonEpisodes(season, media).forEach(e -> service.updateWatchedStated(e, true));
    }

    private void unmarkSeasonAsWatched(Season season) {
        showHelperService.getSeasonEpisodes(season, media).forEach(e -> service.updateWatchedStated(e, false));
    }

    private void selectUnwatchedSeason() {
        var seasons = this.seasons.getItems();
        var season = showHelperService.getUnwatchedSeason(seasons, media);

        Platform.runLater(() -> this.seasons.setSelectedItem(season));
    }

    private void selectUnwatchedEpisode(Season newSeason) {
        var episodes = this.episodes.getItems();
        var episode = showHelperService.getUnwatchedEpisode(episodes, newSeason);

        // check if the current season should be marked as watched
        Platform.runLater(() -> this.episodes.setSelectedItem(episode, true));
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

    private static DetailsComponentListener episodeWatchStateListener(Episode item, EpisodeComponent controller) {
        return new DetailsComponentListener() {
            @Override
            public void onWatchChanged(String imdbId, boolean newState) {
                if (imdbId.equals(item.getId())) {
                    controller.updateWatchedState(newState);
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
        //        MediaTorrentInfo torrentInfo = episode.getTorrents().get(quality);
        //
        //        if (event.getButton() == MouseButton.SECONDARY) {
        //            copyMagnetLink(torrentInfo);
        //        } else {
        //            openMagnetLink(torrentInfo);
        //        }
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
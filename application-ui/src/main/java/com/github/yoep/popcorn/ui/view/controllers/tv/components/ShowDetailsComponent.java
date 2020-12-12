package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.media.watched.controls.WatchedCell;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnTvMode;
import com.github.yoep.popcorn.ui.view.controls.Episodes;
import com.github.yoep.popcorn.ui.view.controls.HorizontalBar;
import com.github.yoep.popcorn.ui.view.models.Season;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.ShowHelperService;
import javafx.application.Platform;
import javafx.beans.value.ChangeListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import javafx.scene.input.InputEvent;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.collections4.CollectionUtils;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

import java.net.URL;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
@ConditionalOnTvMode
public class ShowDetailsComponent extends AbstractTvDetailsComponent<Show> implements Initializable {
    private static final double POSTER_WIDTH = 298.0;
    private static final double POSTER_HEIGHT = 315.0;

    private final ShowHelperService showHelperService;
    private final ChangeListener<Boolean> watchedListener = createWatchListener();

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
    private HorizontalBar<Season> seasons;
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
    private Icon watchedButtonIcon;
    @FXML
    private Label watchedButtonText;
    @FXML
    private Pane episodeDetails;


    //region Constructors

    public ShowDetailsComponent(LocaleText localeText,
                                HealthService healthService,
                                ImageService imageService,
                                SettingsService settingsService,
                                ShowHelperService showHelperService,
                                WatchedService watchedService,
                                ApplicationEventPublisher eventPublisher,
                                SubtitleService subtitleService) {
        super(localeText, imageService, healthService, settingsService, eventPublisher, subtitleService, watchedService);
        this.showHelperService = showHelperService;
    }

    //endregion

    //region Methods

    @EventListener
    public void onShowSerieDetails(ShowSerieDetailsEvent event) {
        Platform.runLater(() -> load(event.getMedia()));
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        super.initialize(location, resources);
        initializeSeasons();
        initializeEpisodes();
    }

    private void initializeSeasons() {
        seasons.setItemFactory(season -> {
            var node = new Label(season.getText());

            node.setOnMouseClicked(e -> onSeasonEvent(e, season));
            node.setOnKeyPressed(e -> onSeasonKeyPressed(e, season));

            return node;
        });
        seasons.selectedItemProperty().addListener((observable, oldValue, newValue) -> onSeasonChanged(newValue));
    }

    private void initializeEpisodes() {
        episodes.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onEpisodeChanged(newValue));
        episodes.setWatchedFactory(() -> new WatchedCell<>() {
            @Override
            protected void onItemChanged(Episode oldItem, Episode newItem) {
                super.onItemChanged(oldItem, newItem);

                if (newItem != null) {
                    boolean watched = watchedService.isWatched(getWatchableItem());

                    setWatched(watched);
                    updateIcon(watched);
                }
            }
        });
    }

    //endregion

    //region AbstractTvDetailsComponent

    @Override
    protected void load(Show media) {
        super.load(media);

        loadText();
        loadSeasons();
    }

    @Override
    protected CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return imageService.loadPoster(media, POSTER_WIDTH, POSTER_HEIGHT);
    }

    @Override
    protected CompletableFuture<List<SubtitleInfo>> retrieveSubtitles() {
        return subtitleService.retrieveSubtitles(media, episode);
    }

    @Override
    protected void reset() {
        super.reset();

        this.title.setText(null);
        this.year.setText(null);
        this.duration.setText(null);
        this.seasons.getItems().clear();
    }

    //endregion

    //region Functions

    @Override
    protected void loadHealth(String quality) {
        switchHealth(episode.getTorrents().get(quality));
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
        seasons.getItems().clear();
        seasons.getItems().addAll(showHelperService.getSeasons(media));
        selectUnwatchedSeason();
    }

    private void loadQualities() {
        var qualities = getVideoResolutions(episode.getTorrents());
        var defaultQuality = getDefaultVideoResolution(qualities);

        Platform.runLater(() -> {
            qualityList.getItems().clear();
            qualityList.getItems().addAll(qualities);
            qualityList.getSelectionModel().select(defaultQuality);
        });
    }

    private void onSeasonChanged(Season season) {
        var episodes = showHelperService.getSeasonEpisodes(season, media);

        this.episodes.getItems().clear();
        this.episodes.getItems().addAll(episodes);
        this.episodeDetails.setVisible(CollectionUtils.isNotEmpty(episodes));

        selectUnwatchedEpisode();
    }

    private void onEpisodeChanged(Episode episode) {
        // remove listeners from the old one
        if (this.episode != null) {
            this.episode.watchedProperty().removeListener(watchedListener);
        }

        this.episode = episode;

        if (episode == null)
            return;

        var airDateTime = episode.getAirDate();

        episodeTitle.setText(episode.getTitle());
        episodeSeason.setText(localeText.get(DetailsMessage.EPISODE_SEASON, episode.getSeason(), episode.getEpisode()));
        airDate.setText(localeText.get(DetailsMessage.AIR_DATE, ShowHelperService.AIRED_DATE_PATTERN.format(airDateTime)));
        episodeOverview.setText(episode.getSynopsis());

        loadQualities();
        loadWatched();
        loadSubtitles();
    }

    private void loadWatched() {
        var watched = watchedService.isWatched(episode);

        episode.setWatched(watched);
        episode.watchedProperty().addListener(watchedListener);
        updateWatched(watched);
    }

    private void updateWatched(boolean newValue) {
        if (newValue) {
            watchedButtonIcon.setText(Icon.EYE_UNICODE);
            watchedButtonText.setText(localeText.get(DetailsMessage.UNMARK_AS_WATCHED));
        } else {
            watchedButtonIcon.setText(Icon.EYE_SLASH_UNICODE);
            watchedButtonText.setText(localeText.get(DetailsMessage.MARK_AS_WATCHED));
        }
    }

    private void selectUnwatchedSeason() {
        var seasons = this.seasons.getItems();
        var season = showHelperService.getUnwatchedSeason(seasons, media);

        Platform.runLater(() -> this.seasons.select(season));
    }

    private void selectUnwatchedEpisode() {
        var episodes = this.episodes.getItems();
        var episode = showHelperService.getUnwatchedEpisode(episodes);

        // check if an episode can be selected
        // if not, end the function as we don't need to select an episode/change focus
        if (episode == null)
            return;

        Platform.runLater(() -> {
            this.episodes.getSelectionModel().select(episode);
            this.episodes.scrollTo(episode);
            this.episodes.requestFocus();
        });
    }

    private void onSeasonEvent(InputEvent event, Season season) {
        event.consume();
        seasons.select(season);
    }

    private void onSeasonKeyPressed(KeyEvent event, Season season) {
        if (event.getCode() == KeyCode.ENTER) {
            onSeasonEvent(event, season);
        }
    }

    private void onWatchNowEvent() {
        var mediaTorrentInfo = episode.getTorrents().get(quality);
        eventPublisher.publishEvent(new LoadMediaTorrentEvent(this, mediaTorrentInfo, episode, quality, null));
    }

    private void onQualityEvent() {
        overlay.show(qualityButton, qualityList);
    }

    private void onWatchedEvent() {
        if (episode == null) {
            log.warn("Unable to update watched state, episode is unknown");
            return;
        }

        if (!episode.isWatched()) {
            watchedService.addToWatchList(episode);
        } else {
            watchedService.removeFromWatchList(episode);
        }
    }

    private void onCloseEvent() {
        eventPublisher.publishEvent(new CloseDetailsEvent(this));
    }

    private ChangeListener<Boolean> createWatchListener() {
        return (observable, oldValue, newValue) -> updateWatched(newValue);
    }

    @FXML
    private void onShowDetailsKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.UNDEFINED) {
            event.consume();
            eventPublisher.publishEvent(new CloseDetailsEvent(this));
        }
    }

    @FXML
    private void onWatchNowClicked(MouseEvent event) {
        event.consume();
        onWatchNowEvent();
    }

    @FXML
    private void onWatchNowKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onWatchNowEvent();
        }
    }

    @FXML
    private void onQualityClicked(MouseEvent event) {
        event.consume();
        onQualityEvent();
    }

    @FXML
    private void onQualityKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onQualityEvent();
        }
    }

    @FXML
    private void onWatchedClicked(MouseEvent event) {
        event.consume();
        onWatchedEvent();
    }

    @FXML
    private void onWatchedKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onWatchedEvent();
        }
    }

    @FXML
    private void onCloseClicked(MouseEvent event) {
        event.consume();
        onCloseEvent();
    }

    @FXML
    private void onCloseKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onCloseEvent();
        }
    }

    //endregion
}

package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Torrent;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.media.filters.model.Season;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.MediaQualityChangedEvent;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controls.*;
import com.github.yoep.popcorn.ui.view.services.*;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class ShowDetailsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    @Mock
    private HealthService healthService;
    @Mock
    private ISubtitleService subtitleService;
    @Mock
    private SubtitlePickerService subtitlePickerService;
    @Mock
    private ImageService imageService;
    @Mock
    private ApplicationConfig settingsService;
    @Mock
    private DetailsComponentService service;
    @Mock
    private ShowHelperService showHelperService;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private SerieActionsComponent serieActionsComponent;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private ShowDetailsComponent component;

    @BeforeEach
    void setUp() {
        component.seasons = new AxisItemSelection<>();
        component.episodes = new AxisItemSelection<>();
        component.episodeDetails = new GridPane();
        component.showDetails = new GridPane();
        component.showDetails.getColumnConstraints().add(0, new ColumnConstraints());
        component.title = new Label("title");
        component.overview = new Label("overview");
        component.year = new Label("year");
        component.duration = new Label("duration");
        component.status = new Label("status");
        component.genres = new Label("genres");
        component.backgroundImage = new BackgroundImageCover();
        component.ratingStars = new Stars();
        component.episodeDetailsOverlay = new Overlay();
        component.health = new HealthIcon();

        lenient().when(viewLoader.load(ShowDetailsComponent.SERIE_ACTIONS_COMPONENT_FXML)).thenReturn(new Pane());
        lenient().when(viewLoader.load(ShowDetailsComponent.POSTER_COMPONENT_FXML)).thenReturn(new Pane());
        lenient().when(viewLoader.load(eq(ShowDetailsComponent.EPISODE_COMPONENT_FXML), isA(EpisodeComponent.class))).thenAnswer(invocations -> new Pane());
        lenient().when(viewLoader.load(ShowDetailsComponent.EPISODE_ACTIONS_COMPONENT_FXML)).thenReturn(new Pane());
        lenient().when(imageService.loadFanart(isA(Media.class))).thenReturn(new CompletableFuture<>());
    }

    @Test
    void testOnShowSerieDetailsEvent_shouldHideEmptySeasons() {
        var media = createShow();
        var season1 = new Season(1, "1");
        var season2 = new Season(2, "2");
        var season3 = new Season(3, "3");
        when(showHelperService.getSeasons(media)).thenReturn(asList(season1, season2, season3));
        when(showHelperService.getSeasonEpisodes(season1, media)).thenReturn(asList(
                new Episode(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Episode.newBuilder()
                        .setSeason(1)
                        .setEpisode(1)
                        .build()),
                new Episode(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Episode.newBuilder()
                        .setSeason(1)
                        .setEpisode(2)
                        .build())));
        when(showHelperService.getSeasonEpisodes(season2, media)).thenReturn(Collections.emptyList());
        when(showHelperService.getSeasonEpisodes(season3, media)).thenReturn(Collections.singletonList(
                new Episode(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Episode.newBuilder()
                        .setSeason(3)
                        .setEpisode(1)
                        .build())));
        when(showHelperService.getUnwatchedSeason(isA(List.class), eq(media))).thenReturn(CompletableFuture.completedFuture(new Season(1, "1")));
        when(showHelperService.getUnwatchedEpisode(isA(List.class), isA(Season.class))).thenReturn(CompletableFuture.completedFuture(media.getEpisodes().getFirst()));
        when(service.isWatched(isA(Media.class))).thenReturn(CompletableFuture.completedFuture(false));
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        assertTrue(component.seasons.getItems().stream().anyMatch(e -> e.season() == 1), "expected season 1 to be present");
        assertFalse(component.seasons.getItems().stream().anyMatch(e -> e.season() == 2), "expected season 2 to not be present");
        assertTrue(component.seasons.getItems().stream().anyMatch(e -> e.season() == 3), "expected season 3 to be present");
    }

    @Test
    void testMediaQualityChangedEvent() {
        var media = createShow();
        var torrentUrl = "myTorrentMagnetUrlThingy";
        var episode = com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Episode.newBuilder()
                .setTorrents(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.TorrentQuality.newBuilder()
                        .putAllQualities(new HashMap<>() {{
                            put("720p", com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.TorrentInfo.newBuilder()
                                    .setSeeds(20)
                                    .setPeers(2)
                                    .setUrl(torrentUrl)
                                    .build());
                        }})
                        .build())
                .build();
        when(healthService.calculateHealth(20, 2)).thenReturn(CompletableFuture.completedFuture(Torrent.Health.newBuilder()
                .setState(Torrent.Health.State.GOOD)
                .build()));
        when(healthService.getTorrentHealth(isA(String.class))).thenReturn(new CompletableFuture<>());
        when(showHelperService.getUnwatchedSeason(isA(List.class), eq(media))).thenReturn(CompletableFuture.completedFuture(new Season(1, "1")));
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();
        component.episode = new Episode(episode);
        eventPublisher.publish(new MediaQualityChangedEvent(this, new Episode(episode), "720p"));
        WaitForAsyncUtils.waitForFxEvents();

        verify(healthService).calculateHealth(20, 2);
        verify(healthService).getTorrentHealth(torrentUrl);
    }

    @Test
    void testOnWatchedClicked() {
        var media = createShow();
        var event = mock(MouseEvent.class);
        when(showHelperService.getUnwatchedSeason(isA(List.class), eq(media))).thenReturn(CompletableFuture.completedFuture(new Season(1, "1")));
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();
        component.onWatchedClicked(event);

        verify(event).consume();
        verify(service).toggleWatchedState(media);
    }

    @Test
    void testOnFavoriteClicked() {
        var media = createShow();
        var event = mock(MouseEvent.class);
        when(showHelperService.getUnwatchedSeason(isA(List.class), eq(media))).thenReturn(CompletableFuture.completedFuture(new Season(1, "1")));
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();
        component.onFavoriteClicked(event);

        verify(event).consume();
        verify(service).toggleLikedState(media);
    }

    private static ShowDetails createShow() {
        return new ShowDetails(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.ShowDetails.newBuilder()
                .setImdbId("tt00001")
                .setTitle("MyShow")
                .setNumberOfSeasons(1)
                .addEpisodes(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Episode.newBuilder()
                        .setSeason(1)
                        .setEpisode(1)
                        .build())
                .build());
    }
}
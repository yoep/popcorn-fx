package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentHealthState;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.filters.model.Season;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
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
    private SubtitleService subtitleService;
    @Mock
    private SubtitlePickerService subtitlePickerServic;
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

        when(viewLoader.load(ShowDetailsComponent.SERIE_ACTIONS_COMPONENT_FXML)).thenReturn(new Pane());
        when(viewLoader.load(ShowDetailsComponent.POSTER_COMPONENT_FXML)).thenReturn(new Pane());
        when(viewLoader.load(ShowDetailsComponent.EPISODE_ACTIONS_COMPONENT_FXML)).thenReturn(new Pane());
        when(imageService.loadFanart(isA(Media.class))).thenReturn(new CompletableFuture<>());
    }

    @Test
    void testOnShowSerieDetailsEvent_shouldHideEmptySeasons() {
        var media = mock(ShowDetails.class);
        var season1 = new Season(1, "1");
        var season2 = new Season(2, "2");
        var season3 = new Season(3, "3");
        when(showHelperService.getSeasons(media)).thenReturn(asList(season1, season2, season3));
        when(showHelperService.getSeasonEpisodes(season1, media)).thenReturn(asList(Episode.builder()
                        .season(1)
                        .episode(1)
                        .build(),
                Episode.builder()
                        .season(1)
                        .episode(2)
                        .build()));
        when(showHelperService.getSeasonEpisodes(season2, media)).thenReturn(Collections.emptyList());
        when(showHelperService.getSeasonEpisodes(season3, media)).thenReturn(Collections.singletonList(Episode.builder()
                .season(3)
                .episode(1)
                .build()));
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        assertTrue(component.seasons.getItems().stream().anyMatch(e -> e.getSeason() == 1), "expected season 1 to be present");
        assertFalse(component.seasons.getItems().stream().anyMatch(e -> e.getSeason() == 2), "expected season 2 to not be present");
        assertTrue(component.seasons.getItems().stream().anyMatch(e -> e.getSeason() == 3), "expected season 3 to be present");
    }

    @Test
    void testMediaQualityChangedEvent() {
        var media = mock(ShowDetails.class);
        var episode = mock(Episode.class);
        var torrentUrl = "myTorrentMagnetUrlThingy";
        var torrentHealth = mock(TorrentHealth.class);
        when(episode.getTorrents()).thenReturn(new HashMap<>(){{
            put("720p", MediaTorrentInfo.builder()
                    .seed(20)
                    .peer(2)
                    .url(torrentUrl)
                    .build());
        }});
        when(healthService.calculateHealth(20, 2)).thenReturn(torrentHealth);
        when(healthService.getTorrentHealth(isA(String.class))).thenReturn(new CompletableFuture<>());
        when(torrentHealth.getState()).thenReturn(TorrentHealthState.GOOD);
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();
        component.episode = episode;
        eventPublisher.publish(new MediaQualityChangedEvent(this, episode, "720p"));
        WaitForAsyncUtils.waitForFxEvents();

        verify(healthService).calculateHealth(20, 2);
        verify(healthService).getTorrentHealth(torrentUrl);
    }

    @Test
    void testOnWatchedClicked() {
        var media = mock(ShowDetails.class);
        var event = mock(MouseEvent.class);
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();
        component.onWatchedClicked(event);

        verify(event).consume();
        verify(service).toggleWatchedState(media);
    }

    @Test
    void testOnFavoriteClicked() {
        var media = mock(ShowDetails.class);
        var event = mock(MouseEvent.class);
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();
        component.onFavoriteClicked(event);

        verify(event).consume();
        verify(service).toggleLikedState(media);
    }
}
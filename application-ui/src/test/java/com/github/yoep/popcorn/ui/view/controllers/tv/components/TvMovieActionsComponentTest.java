package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import javafx.scene.control.Button;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
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
import java.util.HashMap;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvMovieActionsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private LocaleText localeText;
    @Mock
    private DetailsComponentService detailsComponentService;
    @Mock
    private URL location;
    @Mock
    private ResourceBundle resources;
    @InjectMocks
    private TvMovieActionsComponent component;

    @BeforeEach
    void setUp() {
        var none = mock(SubtitleInfo.class);
        when(none.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        lenient().when(subtitleService.none()).thenReturn(none);

        component.watchNowButton = new Button();
        component.watchTrailerButton = new Button();
        component.favoriteButton = new Button();
        component.favoriteIcon = new Icon();
        component.qualities = new AxisItemSelection<>();
        component.qualityOverlay = new Overlay();
        component.subtitles = new AxisItemSelection<>();
        component.subtitleOverlay = new Overlay();
    }

    @Test
    void testOnFavoriteClicked() {
        var event = mock(MouseEvent.class);
        var media = mock(MovieDetails.class);
        when(media.getTorrents()).thenReturn(new HashMap<>() {{
            put(TvMovieActionsComponent.DEFAULT_TORRENT_AUDIO, new HashMap<>());
        }});
        when(subtitleService.retrieveSubtitles(media)).thenReturn(new CompletableFuture<>());
        component.initialize(location, resources);
        eventPublisher.publishEvent(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.onFavoriteClicked(event);

        verify(event).consume();
        verify(detailsComponentService).toggleLikedState(media);
    }

    @Test
    void testOnFavoriteKeyPressed() {
        var event = mock(KeyEvent.class);
        var media = mock(MovieDetails.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(media.getTorrents()).thenReturn(new HashMap<>() {{
            put(TvMovieActionsComponent.DEFAULT_TORRENT_AUDIO, new HashMap<>());
        }});
        when(subtitleService.retrieveSubtitles(media)).thenReturn(new CompletableFuture<>());
        component.initialize(location, resources);
        eventPublisher.publishEvent(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.onFavoritePressed(event);

        verify(event).consume();
        verify(detailsComponentService).toggleLikedState(media);
    }
}
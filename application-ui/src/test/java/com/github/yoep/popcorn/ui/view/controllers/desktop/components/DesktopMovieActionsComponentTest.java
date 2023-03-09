package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.WatchNowEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableMap;
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

import java.net.URL;
import java.util.LinkedHashMap;
import java.util.ResourceBundle;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopMovieActionsComponentTest {
    @Mock
    private PlayerManagerService playerService;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private DesktopMovieActionsComponent component;

    private final ObservableMap<String, Player> playerProperty = FXCollections.observableMap(new LinkedHashMap<>());
    private final ObjectProperty<Player> activePlayerProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        when(playerService.playersProperty()).thenReturn(playerProperty);
        when(playerService.activePlayerProperty()).thenReturn(activePlayerProperty);

        component.watchNowButton = new PlayerDropDownButton();
        component.watchTrailerButton = new Button();
    }

    @Test
    void testWatchNowClicked() {
        var event = mock(MouseEvent.class);
        component.initialize(url, resourceBundle);

        component.onWatchNowClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(isA(WatchNowEvent.class));
    }

    @Test
    void testWatchNowPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        component.initialize(url, resourceBundle);

        component.onWatchNowPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(isA(WatchNowEvent.class));
    }

    @Test
    void testWatchTrailerClicked() {
        var trailer = "my-movie-trailer";
        var title = "lorem ipsum";
        var event = mock(MouseEvent.class);
        var media = mock(MovieDetails.class);
        when(media.getTrailer()).thenReturn(trailer);
        when(media.getTitle()).thenReturn(title);
        when(media.getImages()).thenReturn(mock(Images.class));
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));

        component.onTrailerClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(PlayVideoEvent.builder()
                .source(component)
                .url(trailer)
                .title(title)
                .subtitlesEnabled(false)
                .build());
    }
}
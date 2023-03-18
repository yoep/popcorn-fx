package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.time.LocalDateTime;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicBoolean;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class EpisodeComponentTest {
    @Mock
    private Episode media;
    @Mock
    private LocaleText localeText;
    @Mock
    private ImageService imageService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    private EpisodeComponent component;

    @BeforeEach
    void setUp() {
        component = new EpisodeComponent(media, localeText, imageService);

        component.graphic = new Pane();
        component.episodeNumber = new Label();
        component.watchedIcon = new Icon();
        component.episodeArt = new ImageCover();
        component.title = new Label();
        component.airDate = new Label();
        component.synopsis = new Label();

        component.watchedIcon.setOnMouseClicked(component::onWatchedClicked);
    }

    @Test
    void testUpdateWatchedState() throws TimeoutException {
        when(media.getAirDate()).thenReturn(LocalDateTime.now());
        component.initialize(url, resourceBundle);

        component.updateWatchedState(true);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.graphic.getStyleClass().contains(EpisodeComponent.WATCHED_STYLE));
        assertEquals(Icon.EYE_UNICODE, component.watchedIcon.getText());
    }

    @Test
    void testOnWatchedIconClicked() {
        var event = mock(MouseEvent.class);
        var result = new AtomicBoolean();
        when(media.getAirDate()).thenReturn(LocalDateTime.now());
        component.setOnWatchClicked(result::set);
        component.initialize(url, resourceBundle);

        component.watchedIcon.getOnMouseClicked().handle(event);

        verify(event).consume();
        assertTrue(result.get(), "expected the newState to be True");
    }

    @Test
    void testOnDestroy() {
        var parent = new Pane();
        var invoked = new AtomicBoolean();
        when(media.getAirDate()).thenReturn(LocalDateTime.now());
        component.setOnDestroy(() -> invoked.set(true));
        component.initialize(url, resourceBundle);
        parent.getChildren().add(component.graphic);

        parent.getChildren().clear();

        assertTrue(invoked.get(), "expected the onDestroy method to have been invoked");
    }
}
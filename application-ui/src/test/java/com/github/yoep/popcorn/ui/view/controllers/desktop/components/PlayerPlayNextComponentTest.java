package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.ui.playnext.PlayNextService;
import com.github.yoep.popcorn.ui.view.controls.SizedImageView;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.beans.property.LongProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleLongProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlayerPlayNextComponentTest {
    @Mock
    private ImageService imageService;
    @Mock
    private PlayNextService playNextService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private PlayerPlayNextComponent component;

    private final ObjectProperty<PlayNextService.NextEpisode> nextEpisodeProperty = new SimpleObjectProperty<>();
    private final LongProperty playingInProperty = new SimpleLongProperty();

    @BeforeEach
    void setUp() {
        lenient().when(playNextService.nextEpisodeProperty()).thenReturn(nextEpisodeProperty);
        lenient().when(playNextService.playingInProperty()).thenReturn(playingInProperty);

        component.playNextPane = new Pane();
        component.playNextPoster = new SizedImageView();
        component.showName = new Label();
        component.episodeTitle = new Label();
        component.episodeNumber = new Label();
        component.playingInCountdown = new Label();
    }

    @Test
    void testOnNextEpisodeChanged() throws TimeoutException {
        var showTitle = "Estla";
        var show = mock(ShowDetails.class);
        var episode = new Episode();
        episode.title = "lorem ipsum dolor";
        episode.episode = 12;
        var nextEpisode = new PlayNextService.NextEpisode(show, episode);
        when(show.getTitle()).thenReturn(showTitle);
        when(imageService.loadPoster(isA(Media.class))).thenReturn(new CompletableFuture<>());
        component.initialize(url, resourceBundle);

        nextEpisodeProperty.set(nextEpisode);

        WaitForAsyncUtils.waitFor(100, TimeUnit.MILLISECONDS, () -> component.showName.getText().equals(showTitle));
        WaitForAsyncUtils.waitFor(100, TimeUnit.MILLISECONDS, () -> component.episodeTitle.getText().equals(episode.title));
        WaitForAsyncUtils.waitFor(100, TimeUnit.MILLISECONDS, () -> component.episodeNumber.getText().equals(String.valueOf(episode.episode)));
        verify(imageService).loadPoster(show);
    }

    @Test
    void testOnPlayNextClicked() {
        var event = mock(MouseEvent.class);
        component.initialize(url, resourceBundle);

        component.onPlayNextClicked(event);

        verify(playNextService).playNextEpisodeNow();
    }

    @Test
    void testOnPlayNextPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        component.initialize(url, resourceBundle);

        component.onPlayNextPressed(event);

        verify(playNextService).playNextEpisodeNow();
    }
}
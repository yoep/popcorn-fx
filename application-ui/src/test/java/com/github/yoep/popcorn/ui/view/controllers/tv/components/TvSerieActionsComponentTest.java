package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.ShowDetails;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.VideoQualityService;
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
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvSerieActionsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    @Mock
    private DetailsComponentService detailsComponentService;
    @Mock
    private VideoQualityService videoQualityService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private TvSerieActionsComponent component;

    private final AtomicReference<DetailsComponentListener> listener = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            listener.set(invocation.getArgument(0));
            return null;
        }).when(detailsComponentService).addListener(isA(DetailsComponentListener.class));

        component.favoriteButton = new Button();
        component.favoriteIcon = new Icon();
    }

    @Test
    void testOnLikedStateChangedToLiked() throws TimeoutException {
        var expectedText = "remove";
        var imdbId = "tt11111";
        var media = mock(ShowDetails.class);
        when(media.getImdbId()).thenReturn(imdbId);
        when(localeText.get(DetailsMessage.REMOVE)).thenReturn(expectedText);
        when(localeText.get(DetailsMessage.ADD)).thenReturn("add");
        when(detailsComponentService.isLiked(media)).thenReturn(false, true);
        component.initialize(url, resourceBundle);

        // update media item
        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        verify(detailsComponentService, timeout(200)).isLiked(media);

        var listener = this.listener.get();
        listener.onLikedChanged(imdbId, true);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.favoriteIcon.getText().equals(Icon.HEART_UNICODE));
        verify(detailsComponentService, times(2)).isLiked(media);
        assertEquals(expectedText, component.favoriteButton.getText());
    }

    @Test
    void testOnLikedStateChangedToUnliked() throws TimeoutException {
        var expectedText = "add";
        var imdbId = "tt11111";
        var media = mock(ShowDetails.class);
        when(media.getImdbId()).thenReturn(imdbId);
        when(localeText.get(DetailsMessage.ADD)).thenReturn(expectedText);
        when(localeText.get(DetailsMessage.REMOVE)).thenReturn("remove");
        when(detailsComponentService.isLiked(media)).thenReturn(true, false);
        component.initialize(url, resourceBundle);

        // update media item
        eventPublisher.publish(new ShowSerieDetailsEvent(this, media));
        verify(detailsComponentService, timeout(200)).isLiked(media);

        var listener = this.listener.get();
        listener.onLikedChanged(imdbId, true);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.favoriteIcon.getText().equals(Icon.HEART_O_UNICODE));
        verify(detailsComponentService, times(2)).isLiked(media);
        assertEquals(expectedText, component.favoriteButton.getText());
    }

    @Test
    void testOnFavoriteClicked() {
        var event = mock(MouseEvent.class);
        var show = mock(ShowDetails.class);
        when(detailsComponentService.isLiked(show)).thenReturn(false);
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowSerieDetailsEvent(this, show));

        component.onFavoriteClicked(event);

        verify(event).consume();
        verify(detailsComponentService).toggleLikedState(show);
    }

    @Test
    void testOnFavoritePressed() {
        var event = mock(KeyEvent.class);
        var show = mock(ShowDetails.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(detailsComponentService.isLiked(show)).thenReturn(false);
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowSerieDetailsEvent(this, show));

        component.onFavoritePressed(event);

        verify(event).consume();
        verify(detailsComponentService).toggleLikedState(show);
    }
}
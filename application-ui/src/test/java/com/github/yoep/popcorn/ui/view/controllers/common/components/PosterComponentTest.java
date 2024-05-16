package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Tooltip;
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
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PosterComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private FavoriteService favoriteService;
    @Mock
    private WatchedService watchedService;
    @Mock
    private LocaleText localeText;
    @Mock
    private ImageService imageService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private PosterComponent component;

    private final AtomicReference<FavoriteEventCallback> favoriteCallbackHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            favoriteCallbackHolder.set(invocation.getArgument(0));
            return null;
        }).when(favoriteService).registerListener(isA(FavoriteEventCallback.class));

        component.poster = new ImageCover();
        component.posterHolder = new Pane();
        component.watchedIcon = new Icon("watchedIcon");
        component.watchedTooltip = new Tooltip();
        component.favoriteIcon = new Icon("favoriteIcon");
        component.favoriteTooltip = new Tooltip();
    }

    @Test
    void testOnShowDetailsEvent() throws TimeoutException {
        var event = mock(ShowMovieDetailsEvent.class);
        var media = mock(MovieDetails.class);
        when(event.getMedia()).thenReturn(media);
        when(favoriteService.isLiked(media)).thenReturn(true);
        when(watchedService.isWatched(media)).thenReturn(true);
        when(imageService.loadPoster(media)).thenReturn(new CompletableFuture<>());
        component.initialize(url, resourceBundle);

        eventPublisher.publish(event);
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.favoriteIcon.getStyleClass().contains(PosterComponent.LIKED_STYLE_CLASS));
        assertTrue(component.watchedIcon.getStyleClass().contains(PosterComponent.WATCHED_STYLE_CLASS));
        verify(localeText).get(DetailsMessage.MARK_AS_NOT_SEEN);
        verify(localeText).get(DetailsMessage.REMOVE_FROM_BOOKMARKS);
    }

    @Test
    void testOnLikeStateChanged() throws TimeoutException {
        var imdbId = "tt123444";
        var mediaEvent = mock(ShowMovieDetailsEvent.class);
        var media = mock(MovieDetails.class);
        var event = new FavoriteEvent.ByValue();
        event.tag = FavoriteEvent.Tag.LikedStateChanged;
        event.union = new FavoriteEvent.FavoriteEventCUnion.ByValue();
        event.union.liked_state_changed = new FavoriteEvent.LikedStateChangedBody();
        event.union.liked_state_changed.imdbId = imdbId;
        event.union.liked_state_changed.newState = (byte) 1;
        when(mediaEvent.getMedia()).thenReturn(media);
        when(media.getId()).thenReturn(imdbId);
        when(favoriteService.isLiked(media)).thenReturn(false);
        when(imageService.loadPoster(media)).thenReturn(new CompletableFuture<>());
        component.initialize(url, resourceBundle);

        // add a media item to the details
        eventPublisher.publish(mediaEvent);
        verify(favoriteService).isLiked(media);
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.favoriteIcon.getText().equals(Icon.HEART_O_UNICODE));

        var listener = favoriteCallbackHolder.get();
        listener.callback(event);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.favoriteIcon.getText().equals(Icon.HEART_UNICODE));
    }
}
package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FavoriteEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventListener;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
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
import java.util.Optional;
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

    private final AtomicReference<FavoriteEventListener> favoriteListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            favoriteListenerHolder.set(invocation.getArgument(0, FavoriteEventListener.class));
            return null;
        }).when(favoriteService).addListener(isA(FavoriteEventListener.class));

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
        var media = creatMovieMedia();
        var image = new Image(PosterComponentTest.class.getResourceAsStream("/posterholder.png"));
        when(event.getMedia()).thenReturn(media);
        when(favoriteService.isLiked(media)).thenReturn(CompletableFuture.completedFuture(true));
        when(watchedService.isWatched(media)).thenReturn(CompletableFuture.completedFuture(true));
        when(imageService.getPosterPlaceholder(isA(Double.class), isA(Double.class))).thenReturn(CompletableFuture.completedFuture(image));
        when(imageService.loadPoster(media)).thenReturn(CompletableFuture.completedFuture(Optional.of(image)));
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
        var imdbId = "tt2200000";
        var mediaEvent = mock(ShowSerieDetailsEvent.class);
        var media = createShowMedia(imdbId);
        var image = new Image(PosterComponentTest.class.getResourceAsStream("/posterholder.png"));
        when(mediaEvent.getMedia()).thenReturn(media);
        when(watchedService.isWatched(isA(com.github.yoep.popcorn.backend.media.Media.class))).thenReturn(CompletableFuture.completedFuture(false));
        when(favoriteService.isLiked(media)).thenReturn(CompletableFuture.completedFuture(false));
        when(imageService.getPosterPlaceholder(isA(Double.class), isA(Double.class))).thenReturn(CompletableFuture.completedFuture(image));
        when(imageService.loadPoster(media)).thenReturn(CompletableFuture.completedFuture(Optional.of(image)));
        component.initialize(url, resourceBundle);

        // add a media item to the details
        eventPublisher.publish(mediaEvent);
        verify(favoriteService).isLiked(media);
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.favoriteIcon.getText().equals(Icon.HEART_O_UNICODE));

        var listener = favoriteListenerHolder.get();
        listener.onLikedStateChanged(FavoriteEvent.LikedStateChanged.newBuilder()
                .setImdbId(imdbId)
                .setIsLiked(true)
                .build());

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.favoriteIcon.getText().equals(Icon.HEART_UNICODE));
    }

    private static MovieDetails creatMovieMedia() {
        return new MovieDetails(Media.MovieDetails.newBuilder()
                .setImdbId("tt1000000")
                .setTitle("MyMovieTitle")
                .build());
    }

    private static ShowDetails createShowMedia(String imdbId) {
        return new ShowDetails(Media.ShowDetails.newBuilder()
                .setImdbId(imdbId)
                .setTitle("MyShowTitle")
                .build());
    }
}
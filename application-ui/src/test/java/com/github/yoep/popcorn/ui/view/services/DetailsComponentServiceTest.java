package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import javafx.beans.property.BooleanProperty;
import javafx.beans.value.ChangeListener;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class DetailsComponentServiceTest {
    @Mock
    private FavoriteService favoriteService;
    @Mock
    private WatchedService watchedService;
    @InjectMocks
    private DetailsComponentService service;

    @Test
    void testIsWatched_whenInvoked_shouldPassMediaItemToWatchedService() {
        var media = Movie.builder().build();
        var expectedResult = true;
        when(watchedService.isWatched(media)).thenReturn(expectedResult);

        var result = service.isWatched(media);

        verify(watchedService).isWatched(media);
        assertEquals(expectedResult, result);
    }

    @Test
    void testIsLiked_whenInvoked_shouldPassMediaItemToFavoriteService() {
        var media = Movie.builder().build();
        var expectedResult = false;
        when(favoriteService.isLiked(media)).thenReturn(expectedResult);

        var result = service.isLiked(media);

        verify(favoriteService).isLiked(media);
        assertEquals(expectedResult, result);
    }

    @Test
    void testOnShowDetails_whenLastMediaItemIsKnown_shouldUnsubscribeFromLastMediaItem() {
        var watchedProperty = mock(BooleanProperty.class);
        var likedProperty = mock(BooleanProperty.class);
        var lastItem = mock(Movie.class);
        var newItem = Movie.builder()
                .title("My new movie playback")
                .build();
        var previousEvent = ShowMovieDetailsEvent.builder()
                .source(this)
                .media(lastItem)
                .build();
        var newEvent = ShowMovieDetailsEvent.builder()
                .source(this)
                .media(newItem)
                .build();
        when(lastItem.watchedProperty()).thenReturn(watchedProperty);
        when(lastItem.likedProperty()).thenReturn(likedProperty);
        service.onShowDetails(previousEvent);

        service.onShowDetails(newEvent);

        verify(watchedProperty).removeListener(isA(ChangeListener.class));
        verify(likedProperty).removeListener(isA(ChangeListener.class));
    }

    @Test
    void testListeners_whenMovieWatchedStateIsChanged_shouldInvokedOnWatchedChanged() {
        var movie = Movie.builder()
                .build();
        var event = ShowMovieDetailsEvent.builder()
                .source(this)
                .media(movie)
                .build();
        var listener = mock(DetailsComponentListener.class);
        var expectedResult = true;
        service.onShowDetails(event);
        service.addListener(listener);

        movie.setWatched(expectedResult);

        verify(listener).onWatchChanged(expectedResult);
    }

    @Test
    void testListeners_whenShowWatchedStateIsChanged_shouldInvokedOnWatchedChanged() {
        var show = mock(ShowDetails.class);
        var event = ShowSerieDetailsEvent.builder()
                .source(this)
                .media(show)
                .build();
        var listener = mock(DetailsComponentListener.class);
        var expectedResult = true;
        when(show.getEpisodes()).thenReturn(Collections.singletonList(Episode.builder().build()));
        service.onShowDetails(event);
        service.addListener(listener);

        show.setWatched(expectedResult);

        verify(listener).onWatchChanged(expectedResult);
    }

    @Test
    void testListeners_whenMovieLikedStateIsChanged_shouldInvokedOnLikedChanged() {
        var movie = Movie.builder()
                .build();
        var event = ShowMovieDetailsEvent.builder()
                .source(this)
                .media(movie)
                .build();
        var listener = mock(DetailsComponentListener.class);
        var expectedResult = true;
        service.onShowDetails(event);
        service.addListener(listener);

        movie.setLiked(expectedResult);

        verify(listener).onLikedChanged(expectedResult);
    }

    @Test
    void testUpdateWatchedStated_whenMediaItemIsGivenAndIsWatched_shouldAddToWatchlist() {
        var media = mock(Media.class);

        service.updateWatchedStated(media, true);

        verify(watchedService).addToWatchList(media);
    }

    @Test
    void testUpdateWatchedStated_whenMediaItemIsGivenAndMarkAsUnseen_shouldRemoveFromWatchlist() {
        var media = mock(Media.class);

        service.updateWatchedStated(media, false);

        verify(watchedService).removeFromWatchList(media);
    }

    @Test
    void testToggleWatchedState_whenLastItemIsKnownAndStateIsWatched_shouldRemoveFromWatchlist() {
        var movie = Movie.builder()
                .build();
        var event = ShowMovieDetailsEvent.builder()
                .source(this)
                .media(movie)
                .build();
        movie.setWatched(true);
        service.onShowDetails(event);

        service.toggleWatchedState();

        verify(watchedService).removeFromWatchList(movie);
    }

    @Test
    void testToggleWatchedState_whenLastItemIsKnownAndStateIsNotSeen_shouldAddToWatchlist() {
        var movie = Movie.builder()
                .build();
        var event = ShowMovieDetailsEvent.builder()
                .source(this)
                .media(movie)
                .build();
        movie.setWatched(false);
        service.onShowDetails(event);

        service.toggleWatchedState();

        verify(watchedService).addToWatchList(movie);
    }

    @Test
    void testToggleLikedState_whenLastItemIsKnownAndStateIsUnliked_shouldAddToFavorites() {
        var show = mock(ShowDetails.class);
        var event = ShowSerieDetailsEvent.builder()
                .source(this)
                .media(show)
                .build();
        when(show.isLiked()).thenReturn(false);
        service.onShowDetails(event);

        service.toggleLikedState();

        verify(favoriteService).addToFavorites(show);
    }

    @Test
    void testToggleLikedState_whenLastItemIsKnownAndStateIsLiked_shouldRemoveFromFavorites() {
        var show = mock(ShowDetails.class);
        var event = ShowSerieDetailsEvent.builder()
                .source(this)
                .media(show)
                .build();
        when(show.isLiked()).thenReturn(true);
        service.onShowDetails(event);

        service.toggleLikedState();

        verify(favoriteService).removeFromFavorites(show);
    }
}
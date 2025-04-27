package com.github.yoep.popcorn.backend.media.watched;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.MovieOverview;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class WatchedServiceTest {
    @Mock
    private FxChannel fxChannel;
    private WatchedService service;

    private final AtomicReference<FxCallback<WatchedEvent>> subscriptionHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            subscriptionHolder.set((FxCallback<WatchedEvent>) invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(WatchedEvent.class)), isA(Parser.class), isA(FxCallback.class));

        service = new WatchedService(fxChannel);
    }

    @Test
    void testIsWatched() {
        var media = new MovieOverview(createMedia());
        var request = new AtomicReference<GetIsWatchedRequest>();
        when(fxChannel.send(isA(GetIsWatchedRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetIsWatchedRequest.class));
            return CompletableFuture.completedFuture(GetIsWatchedResponse.newBuilder()
                    .setIsWatched(true)
                    .build());
        });

        var result = service.isWatched(media).resultNow();

        verify(fxChannel).send(isA(GetIsWatchedRequest.class), isA(Parser.class));
        assertEquals("tt1888000", request.get().getItem().getMovieOverview().getImdbId());
        assertTrue(result, "expected the media item to have been watched");
    }

    @Test
    void testAddToWatchlist() {
        var media = new MovieOverview(createMedia());
        var request = new AtomicReference<AddToWatchlistRequest>();
        when(fxChannel.send(isA(AddToWatchlistRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, AddToWatchlistRequest.class));
            return CompletableFuture.completedFuture(AddToWatchlistResponse.newBuilder()
                    .setResult(Response.Result.OK)
                    .build());
        });

        service.addToWatchList(media);

        verify(fxChannel).send(isA(AddToWatchlistRequest.class), isA(Parser.class));
        assertEquals("tt1888000", request.get().getItem().getMovieOverview().getImdbId());
    }

    @Test
    void testRemoveFromWatchlist() {
        var media = new MovieOverview(createMedia());
        var request = new AtomicReference<RemoveFromWatchlistRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, RemoveFromWatchlistRequest.class));
            return null;
        }).when(fxChannel).send(isA(RemoveFromWatchlistRequest.class));

        service.removeFromWatchList(media);

        verify(fxChannel).send(isA(RemoveFromWatchlistRequest.class));
        assertEquals("tt1888000", request.get().getItem().getMovieOverview().getImdbId());
    }

    @Test
    void testOnWatchedStateChanged() {
        var watchedStateChanged = WatchedEvent.WatchedStateChanged.newBuilder()
                .setImdbId("tt00000001")
                .setNewState(true)
                .build();
        var event = WatchedEvent.newBuilder()
                .setEvent(WatchedEvent.Event.STATE_CHANGED)
                .setWatchedStateChanged(watchedStateChanged)
                .build();
        var listener = mock(WatchedEventListener.class);

        service.addListener(listener);
        subscriptionHolder.get().callback(event);

        verify(listener).onWatchedStateChanged(watchedStateChanged);
    }

    private static Media.MovieOverview createMedia() {
        return Media.MovieOverview.newBuilder()
                .setImdbId("tt1888000")
                .setTitle("Lorem")
                .build();
    }
}
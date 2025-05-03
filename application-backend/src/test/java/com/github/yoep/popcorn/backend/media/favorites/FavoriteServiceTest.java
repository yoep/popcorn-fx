package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.ShowOverview;
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
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class FavoriteServiceTest {
    @Mock
    private FxChannel fxChannel;
    private FavoriteService service;

    private final AtomicReference<FxCallback<FavoriteEvent>> subscriptionHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            subscriptionHolder.set(invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(FavoriteEvent.class)), isA(Parser.class), isA(FxCallback.class));

        service = new FavoriteService(fxChannel);
    }

    @Test
    void testIsLiked() {
        var media_id = "tt1236666";
        var media = new ShowOverview(Media.ShowOverview.newBuilder()
                .setImdbId(media_id)
                .build());
        var request = new AtomicReference<GetIsLikedRequest>();
        when(fxChannel.send(isA(GetIsLikedRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetIsLikedRequest.class));
            return CompletableFuture.completedFuture(GetIsLikedResponse.newBuilder()
                    .setIsLiked(true)
                    .build());
        });

        var result = service.isLiked(media).resultNow();

        verify(fxChannel).send(isA(GetIsLikedRequest.class), isA(Parser.class));
        assertTrue(result, "expected the media item to have been liked");
        assertEquals(FxChannel.typeFrom(Media.ShowOverview.class), request.get().getItem().getType());
        assertEquals(media_id, request.get().getItem().getShowOverview().getImdbId());
    }

    @Test
    void testAddToFavorites() {
        var media_id = "tt112233";
        var media = new ShowOverview(Media.ShowOverview.newBuilder()
                .setImdbId(media_id)
                .build());
        var request = new AtomicReference<AddFavoriteRequest>();
        when(fxChannel.send(isA(AddFavoriteRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, AddFavoriteRequest.class));
            return CompletableFuture.completedFuture(AddFavoriteResponse.newBuilder()
                    .setResult(Response.Result.OK)
                    .build());
        });

        service.addToFavorites(media);

        verify(fxChannel).send(isA(AddFavoriteRequest.class), isA(Parser.class));
        assertEquals(media_id, request.get().getItem().getShowOverview().getImdbId());
    }

    @Test
    void testRemoveFromFavorites() {
        var media_id = "tt6666";
        var media = new ShowOverview(Media.ShowOverview.newBuilder()
                .setImdbId(media_id)
                .build());
        var request = new AtomicReference<RemoveFavoriteRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, RemoveFavoriteRequest.class));
            return null;
        }).when(fxChannel).send(isA(RemoveFavoriteRequest.class));

        service.removeFromFavorites(media);

        verify(fxChannel).send(isA(RemoveFavoriteRequest.class));
        assertEquals(media_id, request.get().getItem().getShowOverview().getImdbId());
    }

    @Test
    void testRegisterListener() {
        var likedStateChanged = FavoriteEvent.LikedStateChanged.newBuilder().build();
        var event = FavoriteEvent.newBuilder()
                .setEvent(FavoriteEvent.Event.LIKED_STATE_CHANGED)
                .setLikeStateChanged(likedStateChanged)
                .build();
        var listener = mock(FavoriteEventListener.class);

        service.addListener(listener);
        subscriptionHolder.get().callback(event);

        verify(listener).onLikedStateChanged(likedStateChanged);
    }
}
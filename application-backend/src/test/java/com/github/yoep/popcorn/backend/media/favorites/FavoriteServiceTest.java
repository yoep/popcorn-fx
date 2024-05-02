package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.ShowOverview;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class FavoriteServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;

    @Test
    void testIsLiked() {
        var service = new FavoriteService(fxLib, instance);
        var overview = new ShowOverview.ByReference();
        when(fxLib.is_media_liked(eq(instance), isA(MediaItem.ByReference.class))).thenReturn((byte) 1);

        var result = service.isLiked(overview);

        assertTrue(result);
//        verify(fxLib).dispose_media_item(isA(MediaItem.ByReference.class));
    }

    @Test
    void testFavoriteEventCallback() throws ExecutionException, InterruptedException, TimeoutException {
        var eventFuture = new CompletableFuture<FavoriteEvent.ByValue>();
        var listenerHolder = new AtomicReference<FavoriteEventCallback>();
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(1));
            return null;
        }).when(fxLib).register_favorites_event_callback(isA(PopcornFx.class), isA(FavoriteEventCallback.class));

        var service = new FavoriteService(fxLib, instance);
        verify(fxLib).register_favorites_event_callback(eq(instance), isA(FavoriteEventCallback.class));
        service.registerListener(eventFuture::complete);

        var listener = listenerHolder.get();
        var event = new FavoriteEvent.ByValue();
        event.tag = FavoriteEvent.Tag.LikedStateChanged;
        listener.callback(event);

        var result = eventFuture.get(200, TimeUnit.MILLISECONDS);
        assertEquals(event, result, "expected the listener to have been invoked");
    }
}
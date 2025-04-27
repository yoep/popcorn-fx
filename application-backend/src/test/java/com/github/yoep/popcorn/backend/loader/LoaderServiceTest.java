package com.github.yoep.popcorn.backend.loader;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class LoaderServiceTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private EventPublisher eventPublisher;
    private LoaderService service;

    private final AtomicReference<FxCallback<LoaderEvent>> subscriptionHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            subscriptionHolder.set((FxCallback<LoaderEvent>) invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(LoaderEvent.class)), isA(Parser.class), isA(FxCallback.class));

        service = new LoaderService(fxChannel, eventPublisher);
    }

    @Test
    void testLoad() {
        var url = "magnet:?SomeRandomTorrentMagnet";
        var handle = 3344L;
        var request = new AtomicReference<LoaderLoadRequest>();
        when(fxChannel.send(isA(LoaderLoadRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, LoaderLoadRequest.class));
            return CompletableFuture.completedFuture(LoaderLoadResponse.newBuilder()
                    .setHandle(Handle.newBuilder()
                            .setHandle(handle)
                            .build())
                    .build());
        });

        service.load(url);

        verify(fxChannel).send(isA(LoaderLoadRequest.class), isA(Parser.class));
        assertEquals(url, request.get().getUrl());
        assertEquals(handle, service.lastLoaderHandle.getHandle());
    }

    @Test
    void testCancel_whenLoaderHandleIsNotNull() {
        var handle = Handle.newBuilder()
                .setHandle(8877L)
                .build();
        var request = new AtomicReference<LoaderCancelRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, LoaderCancelRequest.class));
            return null;
        }).when(fxChannel).send(isA(LoaderCancelRequest.class));
        service.lastLoaderHandle = handle;

        service.cancel();

        verify(fxChannel).send(isA(LoaderCancelRequest.class));
        assertEquals(handle, request.get().getHandle());
    }

    @Test
    void testCancel_whenLoaderHandleIsNull() {
        service.cancel();

        verify(fxChannel, times(0)).send(isA(LoaderCancelRequest.class));
    }

    @Test
    void testOnLoaderEvent() {
        var loadingStarted = LoaderEvent.LoadingStarted.newBuilder()
                .setHandle(Handle.newBuilder()
                        .setHandle(123L)
                        .build())
                .build();
        var listener = mock(LoaderListener.class);
        service.addListener(listener);

        subscriptionHolder.get().callback(LoaderEvent.newBuilder()
                .setEvent(LoaderEvent.Event.LOADING_STARTED)
                .setLoadingStarted(loadingStarted)
                .build());

        verify(listener).onLoadingStarted(loadingStarted);
    }
}
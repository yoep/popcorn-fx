package com.github.yoep.popcorn.backend.media.tracking;

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
class TraktTrackingServiceTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private TrackingAuthorization authorization;
    private TraktTrackingService service;

    private final AtomicReference<FxCallback<TrackingProviderEvent>> trackingEventListener = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            trackingEventListener.set(invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(TrackingProviderEvent.class)), isA(Parser.class), isA(FxCallback.class));

        service = new TraktTrackingService(fxChannel, authorization);
    }

    @Test
    void testIsAuthorized() {
        var isAuthorized = true;
        var request = new AtomicReference<GetTrackingProviderIsAuthorizedRequest>();
        when(fxChannel.send(isA(GetTrackingProviderIsAuthorizedRequest.class), isA(Parser.class)))
                .thenAnswer(invocations -> {
                    request.set(invocations.getArgument(0, GetTrackingProviderIsAuthorizedRequest.class));
                    return CompletableFuture.completedFuture(GetTrackingProviderIsAuthorizedResponse.newBuilder()
                            .setIsAuthorized(isAuthorized)
                            .build());
                });

        var result = service.isAuthorized().resultNow();

        verify(fxChannel).send(isA(GetTrackingProviderIsAuthorizedRequest.class), isA(Parser.class));
        assertEquals(TraktTrackingService.TRACKING_ID, request.get().getTrackingProviderId());
        assertEquals(isAuthorized, result);
    }

    @Test
    void testAuthorize() {
        var request = new AtomicReference<TrackingProviderAuthorizeRequest>();
        when(fxChannel.send(isA(TrackingProviderAuthorizeRequest.class), isA(Parser.class)))
                .thenAnswer(invocations -> {
                    request.set(invocations.getArgument(0, TrackingProviderAuthorizeRequest.class));
                    return CompletableFuture.completedFuture(TrackingProviderAuthorizeResponse.newBuilder()
                            .setResult(Response.Result.OK)
                            .build());
                });

        service.authorize();

        verify(fxChannel).send(isA(TrackingProviderAuthorizeRequest.class), isA(Parser.class));
        assertEquals(TraktTrackingService.TRACKING_ID, request.get().getTrackingProviderId());
    }

    @Test
    void testDisconnect() {
        service.disconnect();

        verify(fxChannel).send(isA(TrackingProviderDisconnectRequest.class));
    }

    @Test
    void testOnOpenTrackingAuthorizationRequest() {
        var authorizationUri = "https://MyTrackingProvider.com/authorize";

        var listener = trackingEventListener.get();
        listener.callback(TrackingProviderEvent.newBuilder()
                .setEvent(TrackingProviderEvent.Event.OPEN_AUTHORIZATION_URI)
                .setOpenAuthorizationUri(TrackingProviderEvent.OpenAuthorizationUri.newBuilder()
                        .setUri(authorizationUri)
                        .build())
                .build());

        verify(authorization).open(authorizationUri);
    }

    @Test
    void testOnAuthorizationStateChanged() {
        var newState = TrackingProvider.AuthorizationState.AUTHORIZED;
        var listener = mock(TrackingListener.class);
        service.addListener(listener);

        trackingEventListener.get().callback(TrackingProviderEvent.newBuilder()
                .setEvent(TrackingProviderEvent.Event.AUTHORIZATION_STATE_CHANGED)
                .setAuthorizationStateChanged(TrackingProviderEvent.AuthorizationStateChanged.newBuilder()
                        .setState(newState)
                        .build())
                .build());

        verify(listener).onAuthorizationChanged(true);
    }
}
package com.github.yoep.popcorn.backend.media.tracking;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class TraktTrackingServiceTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private AuthorizationOpenCallback callback;
    @InjectMocks
    private TraktTrackingService service;

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
}
package com.github.yoep.popcorn.backend.media.tracking;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.util.concurrent.CompletableFuture;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class TraktTrackingService extends AbstractListenerService<TrackingListener> implements TrackingService {
    static final String TRACKING_ID = "trakt";

    private final FxChannel fxChannel;
    private final AuthorizationOpenCallback authorizationOpenCallback;

    public TraktTrackingService(FxChannel fxChannel, AuthorizationOpenCallback callback) {
        this.fxChannel = fxChannel;
        this.authorizationOpenCallback = callback;
        init();
    }

    @Override
    public CompletableFuture<Boolean> isAuthorized() {
        return fxChannel.send(GetTrackingProviderIsAuthorizedRequest.newBuilder()
                        .setTrackingProviderId(TRACKING_ID)
                        .build(), GetTrackingProviderIsAuthorizedResponse.parser())
                .thenApply(GetTrackingProviderIsAuthorizedResponse::getIsAuthorized);
    }

    @Override
    public void authorize() {
        fxChannel.send(TrackingProviderAuthorizeRequest.newBuilder()
                        .setTrackingProviderId(TRACKING_ID)
                        .build(), TrackingProviderAuthorizeResponse.parser())
                .thenAccept(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        log.info("Tracking provider {} is authorized", TRACKING_ID);
                    } else {
                        log.error("Tracking provider {} failed to authorize, {}", TRACKING_ID, response.getError());
                    }
                });
    }

    @Override
    public void disconnect() {
        fxChannel.send(TrackingProviderDisconnectRequest.newBuilder()
                .setTrackingProviderId(TRACKING_ID)
                .build());
    }

    void init() {
        fxChannel.subscribe(FxChannel.typeFrom(TrackingProviderEvent.class), TrackingProviderEvent.parser(), this::onTrackingProviderEvent);
    }

    private void onTrackingProviderEvent(TrackingProviderEvent event) {
        switch (event.getEvent()) {
            case AUTHORIZATION_STATE_CHANGED -> invokeListeners(listener ->
                    listener.onAuthorizationChanged(event.getAuthorizationStateChanged().getState() == TrackingProvider.AuthorizationState.AUTHORIZED));
        }
    }
}

package com.github.yoep.popcorn.backend.media.tracking;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class TraktTrackingService extends AbstractListenerService<TrackingListener> implements TrackingService {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final AuthorizationOpenCallback authorizationOpenCallback;
    private final TrackingEventCallback callback = createCallback();

    public TraktTrackingService(FxLib fxLib, PopcornFx instance, AuthorizationOpenCallback callback) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.authorizationOpenCallback = callback;
        init();
    }

    @Override
    public boolean isAuthorized() {
        return fxLib.tracking_is_authorized(instance) == 1;
    }

    @Override
    public void authorize() {
        fxLib.tracking_authorize(instance);
    }

    @Override
    public void disconnect() {
        fxLib.tracking_disconnect(instance);
    }

    void init() {
        fxLib.register_tracking_authorization_open(instance, authorizationOpenCallback);
        fxLib.register_tracking_provider_callback(instance, callback);
    }

    private TrackingEventCallback createCallback() {
        return event -> {
            try (event) {
                switch (event.getTag()) {
                    case AUTHORIZATION_STATE_CHANGED -> invokeListeners(listener -> {
                        listener.onAuthorizationChanged(event.getUnion().getAuthorizationStateChanged_body().getState() == 1);
                    });
                }
            } catch (Exception ex) {
                log.error("Failed to invoke tacking listeners, {}", ex.getMessage(), ex);
            }
        };
    }

}

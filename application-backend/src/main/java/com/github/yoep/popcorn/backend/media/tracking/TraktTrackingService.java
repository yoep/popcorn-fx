package com.github.yoep.popcorn.backend.media.tracking;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class TraktTrackingService extends AbstractListenerService<TrackingListener> implements TrackingService {
    private final FxChannel fxChannel;
    private final AuthorizationOpenCallback authorizationOpenCallback;
    private final TrackingEventCallback callback = createCallback();

    public TraktTrackingService(FxChannel fxChannel, AuthorizationOpenCallback callback) {
        this.fxChannel = fxChannel;
        this.authorizationOpenCallback = callback;
        init();
    }

    @Override
    public boolean isAuthorized() {
        return false;
    }

    @Override
    public void authorize() {
        // TODO
    }

    @Override
    public void disconnect() {
        // TODO
    }

    void init() {
        // TODO
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

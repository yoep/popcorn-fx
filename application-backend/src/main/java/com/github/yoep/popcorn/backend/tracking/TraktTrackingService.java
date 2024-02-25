package com.github.yoep.popcorn.backend.tracking;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@ToString
@EqualsAndHashCode
public class TraktTrackingService implements TrackingService {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final AuthorizationOpenCallback callback;

    public TraktTrackingService(FxLib fxLib, PopcornFx instance, AuthorizationOpenCallback callback) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.callback = callback;
        init();
    }

    @Override
    public void authorize() {
        fxLib.tracking_authorize(instance);
    }

    void init() {
        fxLib.register_tracking_authorization_open(instance, callback);
    }
}

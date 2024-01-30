package com.github.yoep.popcorn.backend.loader;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class LoaderService extends AbstractListenerService<LoaderListener> implements LoaderEventCallback {
    private final FxLib fxLib;
    private final PopcornFx instance;

    public LoaderService(FxLib fxLib, PopcornFx instance) {
        this.fxLib = fxLib;
        this.instance = instance;
        init();
    }

    @Override
    public void callback(LoaderEventC.ByValue event) {
        switch (event.getTag()) {
            case StateChanged -> {
                var stateChangedBody = event.getUnion().getStateChanged_body();
                invokeListeners(e -> e.onStateChanged(stateChangedBody.getState()));
            }
        }
    }

    void init() {
        log.debug("Registering loader event callback");
        fxLib.register_loader_callback(instance, this);
    }
}

package com.github.yoep.popcorn.backend.loader;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.LoadingStartedEvent;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class LoaderService extends AbstractListenerService<LoaderListener> implements FxCallback<LoaderEvent> {
    private final FxChannel fxChannel;
    private final EventPublisher eventPublisher;

    Handle lastLoaderHandle;

    public LoaderService(FxChannel fxChannel, EventPublisher eventPublisher) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        Objects.requireNonNull(eventPublisher, "eventPublisher cannot be null");
        this.fxChannel = fxChannel;
        this.eventPublisher = eventPublisher;
        init();
    }

    public void load(String url) {
        Objects.requireNonNull(url, "url cannot be null");
        fxChannel.send(
                LoaderLoadRequest.newBuilder().setUrl(url).build(),
                LoaderLoadResponse.parser()
        ).whenComplete((response, throwable) -> {
            if (throwable == null) {
                lastLoaderHandle = response.getHandle();
            } else {
                log.error("Failed to load url {}", url, throwable);
            }
        });
    }

    public void cancel() {
        if (lastLoaderHandle != null) {
            fxChannel.send(
                    LoaderCancelRequest.newBuilder().setHandle(lastLoaderHandle).build()
            );
        }
    }

    @Override
    public void callback(LoaderEvent event) {
        switch (event.getEvent()) {
            case LOADING_STARTED -> {
                lastLoaderHandle = event.getLoadingStarted().getHandle();
                eventPublisher.publish(new LoadingStartedEvent(this));
                invokeListeners(e -> e.onLoadingStarted(event.getLoadingStarted()));
            }
            case STATE_CHANGED -> invokeListeners(e -> e.onStateChanged(event.getStateChanged().getState()));
            case PROGRESS_CHANGED -> invokeListeners(e -> e.onProgressChanged(event.getProgressChanged().getProgress()));
            case LOADING_ERROR -> invokeListeners(e -> e.onError(event.getLoadingError().getError()));
        }
    }

    void init() {
        log.debug("Registering loader event callback");
        fxChannel.subscribe(FxChannel.typeFrom(LoaderEvent.class), LoaderEvent.parser(), this);
    }
}

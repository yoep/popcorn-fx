package com.github.yoep.popcorn.backend.loader;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.LoadingStartedEvent;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class LoaderService extends AbstractListenerService<LoaderListener> implements LoaderEventCallback {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final EventPublisher eventPublisher;

    Long lastLoaderHandle;

    public LoaderService(FxLib fxLib, PopcornFx instance, EventPublisher eventPublisher) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.eventPublisher = eventPublisher;
        init();
    }

    public void load(String url) {
        Objects.requireNonNull(url, "url cannot be null");
        lastLoaderHandle = fxLib.loader_load(instance, url);
    }

    public void load(TorrentInfo torrentInfo, TorrentFileInfo torrentFileInfo) {
        // TODO: implement
    }

    public void cancel() {
        fxLib.loader_cancel(instance, lastLoaderHandle);
    }

    @Override
    public void callback(LoaderEventC.ByValue event) {
        try (event) {
            switch (event.getTag()) {
                case LOADING_STARTED -> {
                    var loadingStartedBody = event.getUnion().getLoadingStarted_body();
                    lastLoaderHandle = loadingStartedBody.getHandle();
                    eventPublisher.publish(new LoadingStartedEvent(this));
                    invokeListeners(e -> e.onLoadingStarted(loadingStartedBody.getStartedEvent()));
                }
                case STATE_CHANGED -> {
                    var stateChangedBody = event.getUnion().getStateChanged_body();
                    invokeListeners(e -> e.onStateChanged(stateChangedBody.getState()));
                }
                case LOADING_ERROR -> {
                    var loadingErrorBody = event.getUnion().getLoadingError_body();
                    invokeListeners(e -> e.onError(loadingErrorBody.getError()));
                }
                case PROGRESS_CHANGED -> {
                    var progressChangedBody = event.getUnion().getProgressChanged_body();
                    invokeListeners(e -> e.onProgressChanged(progressChangedBody.getLoadingProgress()));
                }
            }
        } catch (Exception ex) {
            log.error("An unexpected error occurred while handling the loader event C, {}", ex.getMessage(), ex);
        }
    }

    void init() {
        log.debug("Registering loader event callback");
        fxLib.register_loader_callback(instance, this);
    }
}

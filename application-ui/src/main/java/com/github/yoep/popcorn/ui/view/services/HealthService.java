package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentHealth;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.LoadingStartedEvent;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import lombok.extern.slf4j.Slf4j;

import java.util.concurrent.CompletableFuture;

@Slf4j
public class HealthService {
    private final FxChannel fxChannel;
    private final EventPublisher eventPublisher;

    CompletableFuture<TorrentHealth> healthFuture;

    public HealthService(FxChannel fxChannel, EventPublisher eventPublisher) {
        this.fxChannel = fxChannel;
        this.eventPublisher = eventPublisher;
        init();
    }

    //region Methods

    public TorrentHealth calculateHealth(int seeds, int leechers) {
//        var health = fxLib.calculate_torrent_health(instance, seeds, leechers);
//        health.close();
//        fxLib.dispose_torrent_health(health);
//        return health;
        return null;
    }

    public CompletableFuture<TorrentHealth> getTorrentHealth(String url) {
        cancelPreviousFutureIfNeeded();

        healthFuture = CompletableFuture.supplyAsync(() -> {
//            try (var result = fxLib.torrent_health_from_uri(instance, url)) {
//                if (result.getTag() == TorrentHealthResult.Tag.Ok) {
//                    return result.getUnion().getOk().value;
//                } else {
//                    throw new TorrentException(result.getUnion().getErr().error);
//                }
//            }
            return null;
        });

        return healthFuture;
    }

    //endregion

    //region Functions

    private void init() {
        eventPublisher.register(LoadingStartedEvent.class, event -> {
            cancelPreviousFutureIfNeeded();
            return event;
        });
        eventPublisher.register(CloseDetailsEvent.class, event -> {
            cancelPreviousFutureIfNeeded();
            return event;
        });
    }

    private void cancelPreviousFutureIfNeeded() {
        if (healthFuture != null && !healthFuture.isDone()) {
            log.trace("Cancelling current health request");
            healthFuture.cancel(true);
            healthFuture = null;
        }
    }

    //endregion
}

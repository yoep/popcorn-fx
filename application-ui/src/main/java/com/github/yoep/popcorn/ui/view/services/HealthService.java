package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.LoadingStartedEvent;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.FxChannelException;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import lombok.extern.slf4j.Slf4j;

import java.util.concurrent.CompletableFuture;

@Slf4j
public class HealthService {
    private final FxChannel fxChannel;
    private final EventPublisher eventPublisher;

    CompletableFuture<Torrent.Health> healthFuture;

    public HealthService(FxChannel fxChannel, EventPublisher eventPublisher) {
        this.fxChannel = fxChannel;
        this.eventPublisher = eventPublisher;
        init();
    }

    //region Methods

    public CompletableFuture<Torrent.Health> calculateHealth(int seeds, int leechers) {
        return fxChannel.send(CalculateTorrentHealthRequest.newBuilder()
                        .setSeeds(seeds)
                        .setLeechers(leechers)
                        .build(), CalculateTorrentHealthResponse.parser())
                .thenApply(CalculateTorrentHealthResponse::getHealth);
    }

    public CompletableFuture<Torrent.Health> getTorrentHealth(String url) {
        cancelPreviousFutureIfNeeded();

        healthFuture = fxChannel.send(TorrentHealthRequest.newBuilder()
                        .setUri(url)
                        .build(), TorrentHealthResponse.parser())
                .thenApply(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        return response.getHealth();
                    } else {
                        throw new FxChannelException("Failed to retrieve torrent health");
                    }
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

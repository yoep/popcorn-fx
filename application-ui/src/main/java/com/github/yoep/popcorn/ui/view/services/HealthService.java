package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentHealth;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.LoadingStartedEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.TorrentSettings;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class HealthService {
    private final TorrentService torrentService;
    private final ApplicationConfig settingsService;
    private final EventPublisher eventPublisher;

    private CompletableFuture<TorrentHealth> healthFuture;

    public HealthService(TorrentService torrentService, ApplicationConfig settingsService, EventPublisher eventPublisher) {
        this.torrentService = torrentService;
        this.settingsService = settingsService;
        this.eventPublisher = eventPublisher;
        init();
    }

    //region Methods

    /**
     * @see TorrentService#calculateHealth(int, int)
     */
    public TorrentHealth calculateHealth(int seed, int peer) {
        return torrentService.calculateHealth(seed, peer);
    }

    /**
     * @see TorrentService#getTorrentHealth(String, java.io.File)
     */
    public CompletableFuture<TorrentHealth> getTorrentHealth(String url) {
        cancelPreviousFutureIfNeeded();
        var torrentSettings = getTorrentSettings();

        healthFuture = torrentService.getTorrentHealth(url, new File(torrentSettings.getDirectory()));

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
        }
    }

    private TorrentSettings getTorrentSettings() {
        return settingsService.getSettings().getTorrentSettings();
    }

    //endregion
}

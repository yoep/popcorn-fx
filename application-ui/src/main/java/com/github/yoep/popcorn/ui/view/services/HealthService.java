package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.model.TorrentHealth;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
@RequiredArgsConstructor
public class HealthService {
    private final TorrentService torrentService;

    private CompletableFuture<TorrentHealth> healthFuture;

    //region Methods

    /**
     * @see TorrentService#calculateHealth(int, int)
     */
    public TorrentHealth calculateHealth(int seed, int peer) {
        return torrentService.calculateHealth(seed, peer);
    }

    /**
     * @see TorrentService#getTorrentHealth(String)
     */
    public CompletableFuture<TorrentHealth> getTorrentHealth(String url) {
        cancelPreviousFutureIfNeeded();

        healthFuture = torrentService.getTorrentHealth(url);

        return healthFuture;
    }

    /**
     * Cancel the current health retrieval when the media details are being closed or the media is being played.
     */
    @EventListener({LoadMediaTorrentEvent.class, CloseDetailsEvent.class})
    public void onCancelHealthRetrieval() {
        cancelPreviousFutureIfNeeded();
    }

    //endregion

    //region Functions

    private void cancelPreviousFutureIfNeeded() {
        if (healthFuture != null && !healthFuture.isDone()) {
            log.trace("Cancelling current health request");
            healthFuture.cancel(true);
        }
    }

    //endregion
}
